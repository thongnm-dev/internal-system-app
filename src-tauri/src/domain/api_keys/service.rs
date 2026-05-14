use crate::app::result::AppResult;
use crate::domain::api_keys::models::ApiKeyApplication;
use crate::infrastructure::database::connection::open_database;
use std::path::Path;

pub fn list_api_key_applications(app_data_dir: &Path) -> AppResult<Vec<ApiKeyApplication>> {
    let connection = open_database(app_data_dir)?;
    let mut statement = connection.prepare(
        "
        SELECT id, application_name
        FROM api_keys
        WHERE TRIM(application_name) <> ''
        ORDER BY application_name COLLATE NOCASE ASC
        ",
    )?;

    let rows = statement.query_map([], |row| {
        Ok(ApiKeyApplication {
            id: row.get(0)?,
            application_name: row.get(1)?,
        })
    })?;

    let mut applications = Vec::new();
    for row in rows {
        applications.push(row?);
    }

    Ok(applications)
}
