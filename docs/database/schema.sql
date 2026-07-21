-- ============================================================================
-- PJ Yuji Internal Tool — PostgreSQL Database Schema
-- Dialect:   PostgreSQL 15+
-- Idempotent: dùng IF NOT EXISTS / OR REPLACE, chạy lại nhiều lần không lỗi.
-- ============================================================================

-- ============================================================================
-- IDENTITY & ACCESS
-- ============================================================================

CREATE TABLE IF NOT EXISTS users (
    id            SERIAL       PRIMARY KEY,
    username      VARCHAR(100) NOT NULL UNIQUE,
    password_hash TEXT         NOT NULL,
    full_name     VARCHAR(200) NOT NULL DEFAULT '',
    email         VARCHAR(255) NOT NULL DEFAULT '',
    phone         VARCHAR(50)  NOT NULL DEFAULT '',
    address       TEXT         NOT NULL DEFAULT '',
    position      VARCHAR(100) NOT NULL DEFAULT '',
    is_active     BOOLEAN      NOT NULL DEFAULT TRUE,
    created_at    TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at    TIMESTAMPTZ  NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS roles (
    id          SERIAL       PRIMARY KEY,
    name        VARCHAR(50)  NOT NULL UNIQUE,
    description TEXT         NOT NULL DEFAULT '',
    created_at  TIMESTAMPTZ  NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS user_roles (
    user_id     INTEGER     NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    role_id     INTEGER     NOT NULL REFERENCES roles(id) ON DELETE CASCADE,
    assigned_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (user_id, role_id)
);

CREATE TABLE IF NOT EXISTS user_settings (
    user_id    INTEGER     PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
    theme      VARCHAR(10) NOT NULL DEFAULT 'light'
        CHECK (theme IN ('light', 'dark')),
    language   VARCHAR(5)  NOT NULL DEFAULT 'vi'
        CHECK (language IN ('vi', 'en', 'ja')),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- ============================================================================
-- PROJECTS
-- ============================================================================

CREATE TABLE IF NOT EXISTS projects (
    id            SERIAL       PRIMARY KEY,
    code          VARCHAR(20)  NOT NULL UNIQUE,
    name          VARCHAR(200) NOT NULL,
    client        VARCHAR(200) NOT NULL DEFAULT '',
    -- Thông tin Backlog là tùy chọn (có thể không thiết lập) → cho phép NULL.
    project_backlog_key   VARCHAR(20)  DEFAULT '',
    project_backlog_code   numeric,
    project_backlog_name VARCHAR(100) DEFAULT '',
    is_active     BOOLEAN      NOT NULL DEFAULT TRUE,
    created_at    TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at    TIMESTAMPTZ  NOT NULL DEFAULT NOW()
);

-- Migration cho database đã tồn tại: bỏ ràng buộc NOT NULL trên các cột Backlog.
-- Idempotent — chạy lại nhiều lần không gây lỗi.
ALTER TABLE projects ALTER COLUMN project_backlog_key  DROP NOT NULL;
ALTER TABLE projects ALTER COLUMN project_backlog_code DROP NOT NULL;
ALTER TABLE projects ALTER COLUMN project_backlog_name DROP NOT NULL;

CREATE TABLE IF NOT EXISTS project_members (
    id         SERIAL       PRIMARY KEY,
    project_id INTEGER      NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    username   VARCHAR(100) NOT NULL,
    name       VARCHAR(300) NOT NULL,
    UNIQUE(project_id, username)
);

CREATE TABLE IF NOT EXISTS categories (
    id               SERIAL       PRIMARY KEY,
    process_code     VARCHAR(50)  NOT NULL,
    process_name     VARCHAR(200) NOT NULL,
    short_name     VARCHAR(200) NOT NULL,
    display_order    INTEGER      NOT NULL DEFAULT 0,
    is_task_category BOOLEAN      NOT NULL DEFAULT FALSE,
    UNIQUE (process_code)
);

CREATE INDEX IF NOT EXISTS idx_categories_process ON categories(process_code);

CREATE TABLE IF NOT EXISTS project_tasks (
    id            VARCHAR(50)  PRIMARY KEY,
    project_id    INTEGER      NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    short_name    VARCHAR(200) NOT NULL,
    description   TEXT         NOT NULL DEFAULT '',
    assignee      VARCHAR(100) NOT NULL DEFAULT '',
    estimate_hour VARCHAR(20)  NOT NULL DEFAULT '',
    due_date      DATE,
    issue_key     VARCHAR(30)  NOT NULL DEFAULT '',
    is_user_added BOOLEAN      NOT NULL DEFAULT FALSE,
    is_completed  BOOLEAN      NOT NULL DEFAULT FALSE,
    completed_at  TIMESTAMPTZ,
    created_at    TIMESTAMPTZ  NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_project_tasks_project ON project_tasks(project_id);
CREATE INDEX IF NOT EXISTS idx_project_tasks_assignee ON project_tasks(assignee);

CREATE TABLE IF NOT EXISTS project_task_categories (
    task_id     VARCHAR(50) NOT NULL REFERENCES project_tasks(id) ON DELETE CASCADE,
    category_id INTEGER     NOT NULL REFERENCES categories(id) ON DELETE CASCADE,
    PRIMARY KEY (task_id, category_id)
);

-- ============================================================================
-- DAILY WORK
-- ============================================================================

CREATE TABLE IF NOT EXISTS daily_work_notes (
    id         SERIAL       PRIMARY KEY,
    username   VARCHAR(100) NOT NULL,
    content    TEXT         NOT NULL,
    note_date  DATE         NOT NULL,
    status     VARCHAR(15)  NOT NULL DEFAULT 'completed'
        CHECK (status IN ('completed', 'incomplete', 'reserved')),
    created_at TIMESTAMPTZ  NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_daily_notes_user_date
    ON daily_work_notes(username, note_date);

CREATE TABLE IF NOT EXISTS daily_report_entries (
    id          SERIAL           PRIMARY KEY,
    username    VARCHAR(100)     NOT NULL,
    task_id     VARCHAR(120)     NOT NULL,
    project_id  VARCHAR(120)     NOT NULL,
    entry_date  DATE             NOT NULL,
    comment     TEXT             NOT NULL DEFAULT '',
    hour        DOUBLE PRECISION NOT NULL DEFAULT 0,
    is_ot       BOOLEAN          NOT NULL DEFAULT FALSE,
    regular_ot  DOUBLE PRECISION NOT NULL DEFAULT 0,
    midnight_ot DOUBLE PRECISION NOT NULL DEFAULT 0,
    category_id INTEGER          NOT NULL DEFAULT 0,
    updated_at  TIMESTAMPTZ      NOT NULL DEFAULT NOW(),
    UNIQUE(username, task_id, entry_date, category_id)
);

CREATE INDEX IF NOT EXISTS idx_daily_report_entries_user_date
    ON daily_report_entries(username, entry_date);

CREATE TABLE IF NOT EXISTS daily_report_tasks (
    id            SERIAL       PRIMARY KEY,
    username      VARCHAR(100) NOT NULL,
    project_id    VARCHAR(120) NOT NULL,
    name          VARCHAR(300) NOT NULL,
    description   TEXT         NOT NULL DEFAULT '',
    category_id   INTEGER      NOT NULL DEFAULT 0,
    assignee      VARCHAR(100) NOT NULL DEFAULT '',
    estimate_hour VARCHAR(20)  NOT NULL DEFAULT '',
    due_date      VARCHAR(20)  NOT NULL DEFAULT '',
    issue_key     VARCHAR(50)  NOT NULL DEFAULT '',
    is_completed  BOOLEAN      NOT NULL DEFAULT FALSE,
    completed_at  TIMESTAMPTZ,
    created_at    TIMESTAMPTZ  NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_daily_report_tasks_user
    ON daily_report_tasks(username, project_id);

-- ============================================================================
-- DATA IMPORT
-- ============================================================================

CREATE TABLE IF NOT EXISTS import_batches (
    id                SERIAL       PRIMARY KEY,
    source_path       TEXT         NOT NULL,
    source_file_name  VARCHAR(255) NOT NULL,
    report_name       VARCHAR(255) NOT NULL DEFAULT '',
    note              TEXT         NOT NULL DEFAULT '',
    target_month_from VARCHAR(7)   NOT NULL DEFAULT '',
    target_month_to   VARCHAR(7)   NOT NULL DEFAULT '',
    imported_by       VARCHAR(100) NOT NULL DEFAULT '',
    row_count         INTEGER      NOT NULL DEFAULT 0,
    total_minutes     INTEGER      NOT NULL DEFAULT 0,
    imported_at       TIMESTAMPTZ  NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_batches_imported ON import_batches(imported_at);

CREATE TABLE IF NOT EXISTS import_work_records (
    id                              SERIAL       PRIMARY KEY,
    batch_id                        INTEGER      NOT NULL REFERENCES import_batches(id) ON DELETE CASCADE,
    work_date                       DATE         NOT NULL,
    project_code                    VARCHAR(20)  NOT NULL,
    project_name                    VARCHAR(200) NOT NULL,
    process_code                    VARCHAR(20)  NOT NULL,
    process_name                    VARCHAR(200) NOT NULL,
    work_content                    TEXT         NOT NULL DEFAULT '',
    regular_minutes                 INTEGER      NOT NULL DEFAULT 0,
    normal_overtime_minutes         INTEGER      NOT NULL DEFAULT 0,
    legal_holiday_overtime_minutes  INTEGER      NOT NULL DEFAULT 0,
    legal_public_holiday_ot_minutes INTEGER      NOT NULL DEFAULT 0,
    late_night_overtime_minutes     INTEGER      NOT NULL DEFAULT 0,
    total_minutes                   INTEGER      NOT NULL DEFAULT 0
);

CREATE INDEX IF NOT EXISTS idx_records_batch   ON import_work_records(batch_id);
CREATE INDEX IF NOT EXISTS idx_records_project ON import_work_records(project_code);
CREATE INDEX IF NOT EXISTS idx_records_date    ON import_work_records(work_date);

-- ============================================================================
-- SKILLS
-- ============================================================================

CREATE TABLE IF NOT EXISTS skills (
    id          VARCHAR(100) PRIMARY KEY,
    name        VARCHAR(200) NOT NULL,
    category    VARCHAR(30)  NOT NULL DEFAULT 'Other'
        CHECK (category IN ('MCP Tools', 'Prompts', 'Workflows', 'Dev Tools',
                            'Data & APIs', 'Security', 'Automation', 'Other')),
    description TEXT         NOT NULL DEFAULT '',
    publisher   VARCHAR(200) NOT NULL DEFAULT '',
    version     VARCHAR(30)  NOT NULL DEFAULT '0.1.0',
    status      VARCHAR(15)  NOT NULL DEFAULT 'Draft'
        CHECK (status IN ('Active', 'Draft', 'Deprecated')),
    downloads   INTEGER      NOT NULL DEFAULT 0,
    stars       INTEGER      NOT NULL DEFAULT 0,
    installs    INTEGER      NOT NULL DEFAULT 0,
    usage       TEXT         NOT NULL DEFAULT '',
    guidance    TEXT         NOT NULL DEFAULT '',
    created_at  TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ  NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_skills_category ON skills(category);
CREATE INDEX IF NOT EXISTS idx_skills_status   ON skills(status);

CREATE TABLE IF NOT EXISTS skill_tags (
    skill_id VARCHAR(100) NOT NULL REFERENCES skills(id) ON DELETE CASCADE,
    tag      VARCHAR(50)  NOT NULL,
    PRIMARY KEY (skill_id, tag)
);

CREATE INDEX IF NOT EXISTS idx_skill_tags_tag ON skill_tags(tag);

-- ============================================================================
-- GOVERNANCE
-- ============================================================================

CREATE TABLE IF NOT EXISTS menu_configs (
    key           VARCHAR(50)  PRIMARY KEY,
    title         VARCHAR(100) NOT NULL,
    path          VARCHAR(200) NOT NULL,
    icon          VARCHAR(50)  NOT NULL DEFAULT 'pi-circle',
    menu_group    VARCHAR(50)  NOT NULL DEFAULT '—',
    is_visible    BOOLEAN      NOT NULL DEFAULT TRUE,
    display_order INTEGER      NOT NULL DEFAULT 0,
    updated_at    TIMESTAMPTZ  NOT NULL DEFAULT NOW()
);

-- Phân quyền menu theo role: có bản ghi = role được phép truy cập menu đó.
CREATE TABLE IF NOT EXISTS role_menu_permissions (
    role_id    INTEGER     NOT NULL REFERENCES roles(id) ON DELETE CASCADE,
    menu_key   VARCHAR(50) NOT NULL REFERENCES menu_configs(key) ON DELETE CASCADE,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (role_id, menu_key)
);

-- Phân quyền menu riêng cho từng user — ghi đè kết quả tổng hợp từ role.
-- `is_allowed = TRUE` là cấp thêm quyền, `FALSE` là thu hồi quyền role đã cấp.
CREATE TABLE IF NOT EXISTS user_menu_permissions (
    user_id    INTEGER     NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    menu_key   VARCHAR(50) NOT NULL REFERENCES menu_configs(key) ON DELETE CASCADE,
    is_allowed BOOLEAN     NOT NULL DEFAULT TRUE,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (user_id, menu_key)
);

CREATE INDEX IF NOT EXISTS idx_role_menu_perm_menu ON role_menu_permissions(menu_key);
CREATE INDEX IF NOT EXISTS idx_user_menu_perm_menu ON user_menu_permissions(menu_key);

CREATE TABLE IF NOT EXISTS audit_logs (
    id         SERIAL       PRIMARY KEY,
    user_id    INTEGER      REFERENCES users(id) ON DELETE SET NULL,
    username   VARCHAR(100) NOT NULL DEFAULT '',
    action     VARCHAR(50)  NOT NULL,
    entity     VARCHAR(50)  NOT NULL DEFAULT '',
    entity_id  VARCHAR(100) NOT NULL DEFAULT '',
    detail     JSONB        NOT NULL DEFAULT '{}',
    ip_address VARCHAR(45)  NOT NULL DEFAULT '',
    created_at TIMESTAMPTZ  NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_logs_user    ON audit_logs(user_id);
CREATE INDEX IF NOT EXISTS idx_logs_action  ON audit_logs(action);
CREATE INDEX IF NOT EXISTS idx_logs_created ON audit_logs(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_logs_detail  ON audit_logs USING GIN (detail);

-- ============================================================================
-- CLOUD STORAGE
-- ============================================================================
CREATE TABLE IF NOT EXISTS aws_work_folder (
    id SERIAL PRIMARY KEY,
    folder_key VARCHAR(50) NOT NULL,
    name VARCHAR(100) NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS aws_storage (
    id SERIAL PRIMARY KEY,
    code VARCHAR(3) NOT NULL,
    name VARCHAR(100) NOT NULL,
    name_alias VARCHAR(100) NOT NULL,
    subscribe VARCHAR(100) NOT NULL,
    is_upload BOOLEAN DEFAULT FALSE,
    is_download BOOLEAN DEFAULT FALSE,
    file_only BOOLEAN DEFAULT FALSE,
    link_available TEXT[],
    exclude_subscribe TEXT[],
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
-- ============================================================================
-- TRIGGERS — auto-update updated_at
-- ============================================================================

CREATE OR REPLACE FUNCTION update_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE OR REPLACE TRIGGER trg_users_updated_at
    BEFORE UPDATE ON users FOR EACH ROW EXECUTE FUNCTION update_updated_at();

CREATE OR REPLACE TRIGGER trg_projects_updated_at
    BEFORE UPDATE ON projects FOR EACH ROW EXECUTE FUNCTION update_updated_at();

CREATE OR REPLACE TRIGGER trg_skills_updated_at
    BEFORE UPDATE ON skills FOR EACH ROW EXECUTE FUNCTION update_updated_at();

CREATE OR REPLACE TRIGGER trg_user_settings_updated_at
    BEFORE UPDATE ON user_settings FOR EACH ROW EXECUTE FUNCTION update_updated_at();

CREATE OR REPLACE TRIGGER trg_menu_configs_updated_at
    BEFORE UPDATE ON menu_configs FOR EACH ROW EXECUTE FUNCTION update_updated_at();

CREATE OR REPLACE TRIGGER trg_daily_report_entries_updated_at
    BEFORE UPDATE ON daily_report_entries FOR EACH ROW EXECUTE FUNCTION update_updated_at();