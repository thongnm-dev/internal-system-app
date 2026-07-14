-- ============================================================================
-- init_tables.sql
-- Khởi tạo toàn bộ bảng khi ứng dụng khởi động.
-- Idempotent: dùng IF NOT EXISTS, chạy lại nhiều lần không lỗi.
-- ============================================================================

-- Bảng người dùng
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

-- Bảng vai trò
CREATE TABLE IF NOT EXISTS roles (
    id          SERIAL       PRIMARY KEY,
    name        VARCHAR(50)  NOT NULL UNIQUE,
    description TEXT         NOT NULL DEFAULT '',
    created_at  TIMESTAMPTZ  NOT NULL DEFAULT NOW()
);

-- Bảng gán vai trò cho người dùng
CREATE TABLE IF NOT EXISTS user_roles (
    user_id     INTEGER     NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    role_id     INTEGER     NOT NULL REFERENCES roles(id) ON DELETE CASCADE,
    assigned_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (user_id, role_id)
);

-- Bảng danh sách dự án
CREATE TABLE IF NOT EXISTS projects (
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

-- Bảng thành viên dự án (quan hệ N-N giữa project và user)
CREATE TABLE IF NOT EXISTS project_members (
    id         SERIAL       PRIMARY KEY,
    project_id INTEGER      NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    username   VARCHAR(100) NOT NULL,
    name       VARCHAR(300) NOT NULL,
    UNIQUE(project_id, username)
);

-- Bảng danh mục công đoạn (process/phase)
CREATE TABLE IF NOT EXISTS categories (
    id            SERIAL       PRIMARY KEY,
    process_code  VARCHAR(20)  NOT NULL,
    phase_name    VARCHAR(200) NOT NULL,
    display_order INTEGER      NOT NULL DEFAULT 0,
    UNIQUE (process_code)
);

CREATE INDEX IF NOT EXISTS idx_categories_process
    ON categories(process_code);

-- Bảng task của dự án
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
    created_at    TIMESTAMPTZ  NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_project_tasks_project
    ON project_tasks(project_id);
CREATE INDEX IF NOT EXISTS idx_project_tasks_assignee
    ON project_tasks(assignee);

-- Bảng phân loại task (quan hệ N-N giữa task và category)
CREATE TABLE IF NOT EXISTS project_task_categories (
    task_id  VARCHAR(50)  NOT NULL REFERENCES project_tasks(id) ON DELETE CASCADE,
    category VARCHAR(30)  NOT NULL
        CHECK (category IN ('PG', 'Review PG', 'UT', 'Review UT', 'Other')),
    PRIMARY KEY (task_id, category)
);

-- Bảng ghi chú công việc hằng ngày
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

-- Bảng ô nhập giờ công của màn hình daily report
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
    phase       VARCHAR(100)     NOT NULL DEFAULT '',
    updated_at  TIMESTAMPTZ      NOT NULL DEFAULT NOW(),
    UNIQUE(username, task_id, entry_date)
);

CREATE INDEX IF NOT EXISTS idx_daily_report_entries_user_date
    ON daily_report_entries(username, entry_date);

-- Bảng task do người dùng tự thêm trên màn hình daily report
CREATE TABLE IF NOT EXISTS daily_report_tasks (
    id            SERIAL       PRIMARY KEY,
    username      VARCHAR(100) NOT NULL,
    task_id       VARCHAR(120) NOT NULL,
    project_id    VARCHAR(120) NOT NULL,
    code          VARCHAR(50)  NOT NULL DEFAULT 'TASK',
    name          VARCHAR(300) NOT NULL,
    description   TEXT         NOT NULL DEFAULT '',
    categories    TEXT[]       NOT NULL DEFAULT '{}',
    assignee      VARCHAR(100) NOT NULL DEFAULT '',
    estimate_hour VARCHAR(20)  NOT NULL DEFAULT '',
    due_date      VARCHAR(20)  NOT NULL DEFAULT '',
    issue_key     VARCHAR(50)  NOT NULL DEFAULT '',
    created_at    TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    UNIQUE(username, task_id)
);

CREATE INDEX IF NOT EXISTS idx_daily_report_tasks_user
    ON daily_report_tasks(username, project_id);
