# Feature templates — Backend

Companion to [FEATURE_TEMPLATES.md](FEATURE_TEMPLATES.md) — read that first for the why/when. This file is the ready-to-copy backend baseline: models → database store → service → command → stored procedures, all under the fictitious `template`/`Template` domain.

These samples use plain `mod` declarations (`commands/mod.rs` → `pub mod template_commands;`, etc.) — the simplest, most common mechanism. If the target app instead uses a `modules/*.rs` + `#[path]` indirection, register there instead — same rule, different file.

## Models — `src-tauri/src/models/_template.rs`

Rule: DTOs only. Response structs derive `Serialize`; request structs derive `Deserialize`. Field names stay snake_case.

```rust
//! TEMPLATE — not declared in `models/mod.rs`'s module tree, so this file
//! is not compiled. Copy to `<domain>.rs`, rename the three types, add
//! `pub mod <domain>;` to `src-tauri/src/models/mod.rs`.

use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize)]
pub struct TemplateItemSummary {
    pub id: i32,
    pub name: String,
    pub description: String,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Deserialize)]
pub struct CreateTemplateItemRequest {
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateTemplateItemRequest {
    pub name: String,
    pub description: Option<String>,
}
```

## Database store — `src-tauri/src/database/_template_store.rs`

Rule (PostgreSQL apps only — skip this layer entirely if the app has no DB): every query calls a stored procedure, never inline SQL. One `map_row` per struct shape.

```rust
//! TEMPLATE — not declared anywhere, not compiled.
//! Copy to `<domain>_store.rs`, rename functions/SP names, add
//! `pub mod <domain>_store;` to `src-tauri/src/database/mod.rs`, and add the
//! matching `.sql` files under `docs/store-procedure/`.

use crate::app::error::AppError;
use crate::app::result::AppResult;
use crate::models::_template::TemplateItemSummary;
use crate::utils::pgsql_connect;

fn map_row(row: &tokio_postgres::Row) -> TemplateItemSummary {
    TemplateItemSummary {
        id: row.get("id"),
        name: row.get("name"),
        description: row.get("description"),
        created_at: row.get("created_at"),
    }
}

pub async fn list_all() -> AppResult<Vec<TemplateItemSummary>> {
    let client = pgsql_connect::connect().await?;
    let rows = client
        .query("SELECT * FROM sp_template_select_list()", &[])
        .await
        .map_err(|e| AppError::new(format!("Failed to list template items: {e}")))?;
    Ok(rows.iter().map(map_row).collect())
}

pub async fn find_by_id(id: i32) -> AppResult<Option<TemplateItemSummary>> {
    Ok(list_all().await?.into_iter().find(|i| i.id == id))
}

pub async fn insert_item(name: &str, description: &str) -> AppResult<TemplateItemSummary> {
    let client = pgsql_connect::connect().await?;
    let row = client
        .query_one("SELECT * FROM sp_template_insert($1, $2)", &[&name, &description])
        .await
        .map_err(|e| AppError::new(format!("Failed to insert template item: {e}")))?;
    Ok(map_row(&row))
}

pub async fn update_item(id: i32, name: &str, description: &str) -> AppResult<TemplateItemSummary> {
    let client = pgsql_connect::connect().await?;
    let row = client
        .query_opt("SELECT * FROM sp_template_update($1, $2, $3)", &[&id, &name, &description])
        .await
        .map_err(|e| AppError::new(format!("Failed to update template item: {e}")))?
        .ok_or_else(|| AppError::new(format!("Template item '{id}' not found.")))?;
    Ok(map_row(&row))
}

pub async fn delete_by_id(id: i32) -> AppResult<bool> {
    let client = pgsql_connect::connect().await?;
    let row = client
        .query_one("SELECT sp_template_delete($1)", &[&id])
        .await
        .map_err(|e| AppError::new(format!("Failed to delete template item: {e}")))?;
    let deleted: i32 = row.get(0);
    Ok(deleted > 0)
}
```

## Service — `src-tauri/src/services/_template_service.rs`

Rule: validation and business rules live here — never in the command handler, never in the store. The store returns raw data; this layer decides whether an operation is *allowed*.

```rust
//! TEMPLATE — not declared anywhere, not compiled.
//! Copy to `<domain>_service.rs`, rename functions/types, add
//! `pub mod <domain>_service;` to `src-tauri/src/services/mod.rs`.

use crate::app::error::AppError;
use crate::app::result::AppResult;
use crate::database::_template_store;
use crate::models::_template::{CreateTemplateItemRequest, TemplateItemSummary, UpdateTemplateItemRequest};

pub async fn list_items() -> AppResult<Vec<TemplateItemSummary>> {
    _template_store::list_all().await
}

pub async fn create_item(request: CreateTemplateItemRequest) -> AppResult<TemplateItemSummary> {
    let name = request.name.trim().to_string();
    let description = request.description.unwrap_or_default().trim().to_string();
    if name.is_empty() {
        return Err(AppError::new("Item name is required."));
    }
    _template_store::insert_item(&name, &description).await
}

pub async fn update_item(id: i32, request: UpdateTemplateItemRequest) -> AppResult<TemplateItemSummary> {
    let name = request.name.trim().to_string();
    let description = request.description.unwrap_or_default().trim().to_string();
    if name.is_empty() {
        return Err(AppError::new("Item name is required."));
    }
    _template_store::find_by_id(id)
        .await?
        .ok_or_else(|| AppError::new(format!("Template item '{id}' not found.")))?;
    _template_store::update_item(id, &name, &description).await
}

pub async fn delete_item(id: i32) -> AppResult<()> {
    _template_store::find_by_id(id)
        .await?
        .ok_or_else(|| AppError::new(format!("Template item '{id}' not found.")))?;
    if !_template_store::delete_by_id(id).await? {
        return Err(AppError::new(format!("Template item '{id}' not found.")));
    }
    Ok(())
}
```

## Command — `src-tauri/src/commands/_template_commands.rs`

Rule: a command handler does exactly three things — receive the deserialized request, call one service function, map the error with `crate::app::error::log_err`. No business logic, no direct database access here.

```rust
//! TEMPLATE — not declared anywhere, not registered in the invoke handler,
//! so these commands are not compiled/callable.
//!
//! To use:
//! 1. Copy to `<domain>_commands.rs`, rename functions/types.
//! 2. Add `pub mod <domain>_commands;` to `src-tauri/src/commands/mod.rs`.
//! 3. Add every fn to the `tauri::generate_handler![...]` list in `lib.rs`
//!    (or wherever the app centralizes that list).
//! 4. Add matching functions to `src/tauri/commands/<domain>.ts` on the frontend.

use crate::models::_template::{CreateTemplateItemRequest, TemplateItemSummary, UpdateTemplateItemRequest};
use crate::services::_template_service;

#[tauri::command]
pub async fn list_template_items() -> Result<Vec<TemplateItemSummary>, String> {
    _template_service::list_items().await.map_err(crate::app::error::log_err)
}

#[tauri::command]
pub async fn create_template_item(request: CreateTemplateItemRequest) -> Result<TemplateItemSummary, String> {
    _template_service::create_item(request).await.map_err(crate::app::error::log_err)
}

#[tauri::command]
pub async fn update_template_item(item_id: i32, request: UpdateTemplateItemRequest) -> Result<TemplateItemSummary, String> {
    _template_service::update_item(item_id, request).await.map_err(crate::app::error::log_err)
}

#[tauri::command]
pub async fn delete_template_item(item_id: i32) -> Result<(), String> {
    _template_service::delete_item(item_id).await.map_err(crate::app::error::log_err)
}
```

## Stored procedures — `docs/store-procedure/_TEMPLATE.md` (PostgreSQL apps only)

Rule: naming convention `sp_{domain}_{action}.sql`, one procedure per file. Register every new SP in **both** the SP-management listing and the startup auto-install list — an SP missing from either silently doesn't apply.

```sql
-- sp_template_select_list.sql
CREATE OR REPLACE FUNCTION sp_template_select_list()
RETURNS TABLE (id INTEGER, name VARCHAR, description VARCHAR, created_at TIMESTAMP) AS $$
BEGIN
    RETURN QUERY
    SELECT t.id, t.name, t.description, t.created_at
    FROM template_item t
    ORDER BY t.created_at DESC;
END;
$$ LANGUAGE plpgsql;
```

```sql
-- sp_template_insert.sql
CREATE OR REPLACE FUNCTION sp_template_insert(p_name VARCHAR, p_description VARCHAR)
RETURNS TABLE (id INTEGER, name VARCHAR, description VARCHAR, created_at TIMESTAMP) AS $$
BEGIN
    RETURN QUERY
    INSERT INTO template_item (name, description)
    VALUES (p_name, p_description)
    RETURNING template_item.id, template_item.name, template_item.description, template_item.created_at;
END;
$$ LANGUAGE plpgsql;
```

```sql
-- sp_template_update.sql
CREATE OR REPLACE FUNCTION sp_template_update(p_id INTEGER, p_name VARCHAR, p_description VARCHAR)
RETURNS TABLE (id INTEGER, name VARCHAR, description VARCHAR, created_at TIMESTAMP) AS $$
BEGIN
    RETURN QUERY
    UPDATE template_item
    SET name = p_name, description = p_description
    WHERE template_item.id = p_id
    RETURNING template_item.id, template_item.name, template_item.description, template_item.created_at;
END;
$$ LANGUAGE plpgsql;
```

```sql
-- sp_template_delete.sql
CREATE OR REPLACE FUNCTION sp_template_delete(p_id INTEGER)
RETURNS INTEGER AS $$
DECLARE
    v_deleted INTEGER;
BEGIN
    DELETE FROM template_item WHERE id = p_id;
    GET DIAGNOSTICS v_deleted = ROW_COUNT;
    RETURN v_deleted;
END;
$$ LANGUAGE plpgsql;
```

Matching table, added to `docs/database/schema.sql`:

```sql
CREATE TABLE IF NOT EXISTS template_item (
    id SERIAL PRIMARY KEY,
    name VARCHAR(200) NOT NULL,
    description VARCHAR(500) NOT NULL DEFAULT '',
    created_at TIMESTAMP NOT NULL DEFAULT now()
);
```

See [FEATURE_TEMPLATES.md](FEATURE_TEMPLATES.md) for the unregistered-file rules and when to generate these.
