-- ============================================================================
-- seed_data.sql
-- Dữ liệu mặc định khi khởi tạo database.
-- Idempotent: dùng ON CONFLICT DO NOTHING, chạy lại nhiều lần không lỗi.
-- ============================================================================

-- Role mặc định
INSERT INTO roles (name, description)
VALUES ('admin', 'Administrator with full access')
ON CONFLICT (name) DO NOTHING;

INSERT INTO roles (name, description)
VALUES ('manager', 'Manager with project oversight')
ON CONFLICT (name) DO NOTHING;

INSERT INTO roles (name, description)
VALUES ('member', 'Regular team member')
ON CONFLICT (name) DO NOTHING;

INSERT INTO roles (name, description)
VALUES ('viewer', 'Read-only access')
ON CONFLICT (name) DO NOTHING;

-- User admin mặc định (password: ad@123456)
INSERT INTO users (username, password_hash, full_name, position)
VALUES (
    'admin',
    '$2b$12$dTyhgIqskYwXMkSfe6Luyuq0Ve7EMFS7Rrq7Z5eXvx7apv0bk9cOy',
    'Administrator',
    'System Admin'
)
ON CONFLICT (username) DO NOTHING;

-- Gán role admin cho user admin
INSERT INTO user_roles (user_id, role_id)
SELECT u.id, r.id
FROM users u, roles r
WHERE u.username = 'admin' AND r.name = 'admin'
ON CONFLICT (user_id, role_id) DO NOTHING;

-- Công đoạn (process/phase) mặc định — dùng cho dropdown Phase ở màn Daily Report.
INSERT INTO categories (process_code, process_name, display_order)
VALUES
    ('10', 'PG',                1),
    ('11', 'UT',                2),
    ('48', 'Review PG',         3),
    ('49', 'Review UT',         4),
    ('43', 'Bug',               5),
    ('45', 'Thay đổi qui cách', 6),
    ('40', 'Other',             7)
ON CONFLICT (process_code) DO NOTHING;

-- Menu configuration
INSERT INTO menu_configs (key, title, path, icon, menu_group, is_visible, display_order) VALUES
    ('overview',        'Overview',          '/overview',          'pi-home',       '—',          TRUE, 0),
    ('projects',        'Projects',          '/projects',          'pi-table',      '—',          TRUE, 1),
    ('issueBacklog',    'Issue Backlog',     '/issue-backlog',     'pi-list-check', '—',          TRUE, 2),
    ('dailyWorkNotes',  'Daily Work Notes',  '/daily-work-notes',  'pi-pencil',     '—',          TRUE, 3),
    ('dailyReport',     'Daily Report',      '/daily-report',      'pi-calendar',   '—',          TRUE, 4),
    ('excel2md',        'Excel to Markdown', '/excel2md',          'pi-file',       'Tools',      TRUE, 5),
    ('importCsv',       'Import CSV',        '/import-csv',        'pi-database',   'Tools',      TRUE, 6),
    ('projectSkills',   'Skills',            '/project-skills',    'pi-book',       'Governance', TRUE, 7),
    ('governanceMenus', 'Menus',             '/governance/menus',  'pi-bars',       'Governance', TRUE, 8),
    ('governanceUsers', 'Users',             '/governance/users',  'pi-users',      'Governance', TRUE, 9),
    ('governanceLogs',  'Logs',              '/governance/logs',   'pi-history',    'Governance', TRUE, 10),
    ('settings',        'Settings',          '/settings',          'pi-cog',        '—',          TRUE, 11)
ON CONFLICT (key) DO NOTHING;
