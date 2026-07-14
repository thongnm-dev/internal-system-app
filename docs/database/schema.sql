-- ============================================================================
-- PJ Yuji Internal Tool — PostgreSQL Database Schema
-- Generated: 2026-07-13
-- Dialect:   PostgreSQL 15+
-- Tables:    16
-- ============================================================================

-- ============================================================================
-- IDENTITY & ACCESS
-- Source screens: Login, Settings, Governance Users
-- ============================================================================

CREATE TABLE users (
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

CREATE TABLE roles (
    id          SERIAL       PRIMARY KEY,
    name        VARCHAR(50)  NOT NULL UNIQUE,
    description TEXT         NOT NULL DEFAULT '',
    created_at  TIMESTAMPTZ  NOT NULL DEFAULT NOW()
);

CREATE TABLE user_roles (
    user_id     INTEGER     NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    role_id     INTEGER     NOT NULL REFERENCES roles(id) ON DELETE CASCADE,
    assigned_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (user_id, role_id)
);

CREATE TABLE user_settings (
    user_id    INTEGER     PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
    theme      VARCHAR(10) NOT NULL DEFAULT 'light'
        CHECK (theme IN ('light', 'dark')),
    language   VARCHAR(5)  NOT NULL DEFAULT 'vi'
        CHECK (language IN ('vi', 'en', 'ja')),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE api_keys (
    id         VARCHAR(36)  PRIMARY KEY,
    user_id    INTEGER      NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name       VARCHAR(200) NOT NULL,
    key_label  VARCHAR(200) NOT NULL DEFAULT '',
    api_key    TEXT         NOT NULL,
    created_at TIMESTAMPTZ  NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_api_keys_user ON api_keys(user_id);

-- ============================================================================
-- PROJECTS
-- Source screens: Projects, Project Detail, Project Tasks, Daily Report
-- ============================================================================

CREATE TABLE projects (
    id            SERIAL       PRIMARY KEY,
    code          VARCHAR(20)  NOT NULL UNIQUE,
    name          VARCHAR(200) NOT NULL,
    client        VARCHAR(200) NOT NULL DEFAULT '',
    backlog_key   VARCHAR(20)  NOT NULL DEFAULT '',
    backlog_url   TEXT         NOT NULL DEFAULT '',
    backlog_space VARCHAR(100) NOT NULL DEFAULT '',
    is_active     BOOLEAN      NOT NULL DEFAULT TRUE,
    created_at    TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at    TIMESTAMPTZ  NOT NULL DEFAULT NOW()
);

CREATE UNIQUE INDEX idx_projects_backlog_key
    ON projects(backlog_key) WHERE backlog_key != '';

CREATE TABLE project_members (
    id         SERIAL       PRIMARY KEY,
    project_id INTEGER      NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    username   VARCHAR(100) NOT NULL,
    name       VARCHAR(300) NOT NULL,
    UNIQUE (project_id, username)
);

CREATE INDEX idx_members_project ON project_members(project_id);

CREATE TABLE categories (
    id            SERIAL       PRIMARY KEY,
    process_code  VARCHAR(20)  NOT NULL,
    phase_name    VARCHAR(200) NOT NULL,
    display_order INTEGER      NOT NULL DEFAULT 0,
    UNIQUE (process_code)
);

CREATE INDEX idx_categories_process ON categories(process_code);

CREATE TABLE project_tasks (
    id            VARCHAR(50)  PRIMARY KEY,
    project_id    INTEGER      NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    short_name    VARCHAR(200) NOT NULL,
    description   TEXT         NOT NULL DEFAULT '',
    assignee      VARCHAR(100) NOT NULL DEFAULT '',
    estimate_hour VARCHAR(20)  NOT NULL DEFAULT '',
    due_date      DATE,
    issue_key     VARCHAR(30)  NOT NULL DEFAULT '',
    is_user_added BOOLEAN      NOT NULL DEFAULT FALSE,
    created_at    TIMESTAMPTZ  NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_tasks_project  ON project_tasks(project_id);
CREATE INDEX idx_tasks_assignee ON project_tasks(assignee);
CREATE INDEX idx_tasks_issue    ON project_tasks(issue_key);

CREATE TABLE project_task_categories (
    task_id  VARCHAR(50)  NOT NULL REFERENCES project_tasks(id) ON DELETE CASCADE,
    category VARCHAR(30)  NOT NULL
        CHECK (category IN ('PG', 'Review PG', 'UT', 'Review UT', 'Other')),
    PRIMARY KEY (task_id, category)
);

-- ============================================================================
-- ISSUES
-- Source screens: Issue Backlog, Import Issues
-- ============================================================================

CREATE TABLE issues (
    id                  SERIAL       PRIMARY KEY,
    project_id          INTEGER      NOT NULL REFERENCES projects(id),
    issue_key           VARCHAR(30)  NOT NULL UNIQUE,
    issue_type          VARCHAR(30)  NOT NULL
        CHECK (issue_type IN ('Bug', 'Task', 'Story', 'Improvement')),
    subject             TEXT         NOT NULL,
    description         TEXT         NOT NULL DEFAULT '',
    assignee            VARCHAR(100) NOT NULL DEFAULT '',
    status              VARCHAR(20)  NOT NULL DEFAULT 'Open'
        CHECK (status IN ('Open', 'In Progress', 'Review', 'Resolved', 'Closed')),
    priority            VARCHAR(10)  NOT NULL DEFAULT 'Medium'
        CHECK (priority IN ('Critical', 'High', 'Medium', 'Low')),
    category            VARCHAR(30)  NOT NULL DEFAULT ''
        CHECK (category IN ('', 'Backend', 'Frontend', 'Database', 'Integration', 'Operation')),
    bug_class           VARCHAR(30)  NOT NULL DEFAULT ''
        CHECK (bug_class IN ('', 'Functional', 'UI', 'Performance', 'Security', 'Data')),
    estimated_hours     NUMERIC(8,2) NOT NULL DEFAULT 0,
    actual_hours        NUMERIC(8,2) NOT NULL DEFAULT 0,
    start_date          DATE,
    due_date            DATE,
    version             VARCHAR(50)  NOT NULL DEFAULT '',
    milestones          VARCHAR(200) NOT NULL DEFAULT '',
    parent_issue_key    VARCHAR(30),
    bug_types           VARCHAR(100) NOT NULL DEFAULT '',
    bug_severity_levels VARCHAR(100) NOT NULL DEFAULT '',
    test_phase          VARCHAR(100) NOT NULL DEFAULT '',
    create_user         VARCHAR(100) NOT NULL DEFAULT '',
    created_at          TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at          TIMESTAMPTZ  NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_issues_project  ON issues(project_id);
CREATE INDEX idx_issues_status   ON issues(status);
CREATE INDEX idx_issues_assignee ON issues(assignee);
CREATE INDEX idx_issues_type     ON issues(issue_type);
CREATE INDEX idx_issues_priority ON issues(priority);
CREATE INDEX idx_issues_date     ON issues(created_at);

-- ============================================================================
-- DAILY WORK
-- Source screens: Daily Work Notes, Daily Report
-- ============================================================================

CREATE TABLE daily_work_notes (
    id         VARCHAR(50)  PRIMARY KEY,
    user_id    INTEGER      NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    content    TEXT         NOT NULL,
    note_date  DATE         NOT NULL,
    status     VARCHAR(15)  NOT NULL DEFAULT 'completed'
        CHECK (status IN ('completed', 'incomplete', 'reserved')),
    created_at TIMESTAMPTZ  NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_notes_user_date ON daily_work_notes(user_id, note_date);
CREATE INDEX idx_notes_status    ON daily_work_notes(status);

CREATE TABLE daily_report_entries (
    id          SERIAL       PRIMARY KEY,
    user_id     INTEGER      NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    task_id     VARCHAR(50)  NOT NULL REFERENCES project_tasks(id),
    report_date DATE         NOT NULL,
    hour        NUMERIC(4,2) NOT NULL DEFAULT 0,
    phase       VARCHAR(50)  NOT NULL DEFAULT '',
    comment     TEXT         NOT NULL DEFAULT '',
    is_ot       BOOLEAN      NOT NULL DEFAULT FALSE,
    regular_ot  NUMERIC(4,2) NOT NULL DEFAULT 0,
    midnight_ot NUMERIC(4,2) NOT NULL DEFAULT 0,
    updated_at  TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    UNIQUE (user_id, task_id, report_date)
);

CREATE INDEX idx_entries_user_month
    ON daily_report_entries(user_id, date_trunc('month', report_date));
CREATE INDEX idx_entries_task ON daily_report_entries(task_id);

-- ============================================================================
-- DATA IMPORT
-- Source screens: Import CSV, Overview
-- Replaces the JSON file store (pjjyuji_statistics.json / report_store.rs)
-- ============================================================================

CREATE TABLE import_batches (
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

CREATE INDEX idx_batches_imported ON import_batches(imported_at);

CREATE TABLE import_work_records (
    id                              SERIAL       PRIMARY KEY,
    batch_id                        INTEGER      NOT NULL REFERENCES import_batches(id) ON DELETE CASCADE,
    work_date                       DATE         NOT NULL,
    project_code                    VARCHAR(20)  NOT NULL,
    project_name                    VARCHAR(200) NOT NULL,
    process_code                    VARCHAR(20)  NOT NULL,
    phase_name                      VARCHAR(200) NOT NULL,
    work_content                    TEXT         NOT NULL DEFAULT '',
    regular_minutes                 INTEGER      NOT NULL DEFAULT 0,
    normal_overtime_minutes         INTEGER      NOT NULL DEFAULT 0,
    legal_holiday_overtime_minutes  INTEGER      NOT NULL DEFAULT 0,
    legal_public_holiday_ot_minutes INTEGER      NOT NULL DEFAULT 0,
    late_night_overtime_minutes     INTEGER      NOT NULL DEFAULT 0,
    total_minutes                   INTEGER      NOT NULL DEFAULT 0
);

CREATE INDEX idx_records_batch   ON import_work_records(batch_id);
CREATE INDEX idx_records_project ON import_work_records(project_code);
CREATE INDEX idx_records_date    ON import_work_records(work_date);

-- ============================================================================
-- SKILLS
-- Source screen: Skills catalog
-- ============================================================================

CREATE TABLE skills (
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

CREATE INDEX idx_skills_category ON skills(category);
CREATE INDEX idx_skills_status   ON skills(status);

CREATE TABLE skill_tags (
    skill_id VARCHAR(100) NOT NULL REFERENCES skills(id) ON DELETE CASCADE,
    tag      VARCHAR(50)  NOT NULL,
    PRIMARY KEY (skill_id, tag)
);

CREATE INDEX idx_skill_tags_tag ON skill_tags(tag);

-- ============================================================================
-- GOVERNANCE
-- Source screens: Governance Menus, Governance Logs
-- ============================================================================

CREATE TABLE menu_configs (
    key           VARCHAR(50)  PRIMARY KEY,
    title         VARCHAR(100) NOT NULL,
    path          VARCHAR(200) NOT NULL,
    icon          VARCHAR(50)  NOT NULL DEFAULT 'pi-circle',
    menu_group    VARCHAR(50)  NOT NULL DEFAULT '—',
    is_visible    BOOLEAN      NOT NULL DEFAULT TRUE,
    display_order INTEGER      NOT NULL DEFAULT 0,
    updated_at    TIMESTAMPTZ  NOT NULL DEFAULT NOW()
);

CREATE TABLE audit_logs (
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

CREATE INDEX idx_logs_user    ON audit_logs(user_id);
CREATE INDEX idx_logs_action  ON audit_logs(action);
CREATE INDEX idx_logs_created ON audit_logs(created_at DESC);
CREATE INDEX idx_logs_detail  ON audit_logs USING GIN (detail);

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

CREATE TRIGGER trg_users_updated_at
    BEFORE UPDATE ON users FOR EACH ROW EXECUTE FUNCTION update_updated_at();

CREATE TRIGGER trg_projects_updated_at
    BEFORE UPDATE ON projects FOR EACH ROW EXECUTE FUNCTION update_updated_at();

CREATE TRIGGER trg_issues_updated_at
    BEFORE UPDATE ON issues FOR EACH ROW EXECUTE FUNCTION update_updated_at();

CREATE TRIGGER trg_skills_updated_at
    BEFORE UPDATE ON skills FOR EACH ROW EXECUTE FUNCTION update_updated_at();

CREATE TRIGGER trg_user_settings_updated_at
    BEFORE UPDATE ON user_settings FOR EACH ROW EXECUTE FUNCTION update_updated_at();

CREATE TRIGGER trg_menu_configs_updated_at
    BEFORE UPDATE ON menu_configs FOR EACH ROW EXECUTE FUNCTION update_updated_at();

CREATE TRIGGER trg_daily_report_entries_updated_at
    BEFORE UPDATE ON daily_report_entries FOR EACH ROW EXECUTE FUNCTION update_updated_at();
