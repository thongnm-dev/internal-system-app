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
    'Thongnm',
    '$2b$12$dTyhgIqskYwXMkSfe6Luyuq0Ve7EMFS7Rrq7Z5eXvx7apv0bk9cOy',
    'Thongnm',
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
INSERT INTO categories (process_code, process_name, short_name, is_task_category, display_order)
VALUES
    ('10', 'PG',               'PG' ,true, 1),
    ('11', 'UT',               'UT' ,true, 2),
    ('48', 'Review PG',        'RWPG' ,true, 3),
    ('49', 'Review UT',        'RWUT' ,true, 4),
    ('43', 'Bug',              'BUG' ,true, 5),
    ('45', 'Thay đổi qui cách','SPEC' ,true, 6),
    ('40', 'Other',            'OTHER' ,false, 7)
ON CONFLICT (process_code) DO NOTHING;

-- Menu configuration
INSERT INTO menu_configs (key, title, path, icon, menu_group, is_visible, display_order) VALUES
    ('overview',        'Overview',          '/overview',          'pi-home',       '—',          TRUE, 0),
    ('projects',        'Projects',          '/projects',          'pi-table',      '—',          TRUE, 1),
    ('issueBacklog',    'Issue Backlog',     '/issue-backlog',     'pi-list-check', '—',          TRUE, 2),
    ('dailyWorkNotes',  'Daily Work Notes',  '/daily-work-notes',  'pi-pencil',     '—',          TRUE, 3),
    ('dailyReport',     'Daily Report',      '/daily-report',      'pi-calendar',   '—',          TRUE, 4),
    ('excel2md',        'Excel to Markdown', '/excel2md',          'pi-file',       'Tools',      TRUE, 5),
    ('checkMonthlyReport', 'Check Monthly Report', '/check-monthly-report', 'pi-database', 'Tools', TRUE, 6),
    ('projectSkills',   'Skills',            '/project-skills',    'pi-book',       'Governance', TRUE, 7),
    ('governanceMenus', 'Menus',             '/governance/menus',  'pi-bars',       'Governance', TRUE, 8),
    ('governanceUsers', 'Users',             '/governance/users',  'pi-users',      'Governance', TRUE, 9),
    ('governanceLogs',  'Logs',              '/governance/logs',   'pi-history',    'Governance', TRUE, 10),
    ('settings',        'Settings',          '/settings',          'pi-cog',        '—',          TRUE, 11)
ON CONFLICT (key) DO NOTHING;

--- AWS Storage configuration
truncate table aws_work_folder cascade;
truncate table aws_storage cascade;
ALTER SEQUENCE aws_work_folder_id_seq RESTART WITH 1;
ALTER SEQUENCE aws_storage_id_seq RESTART WITH 1;

INSERT INTO public.aws_work_folder (folder_key, "name") VALUES
('CORRECT_BUG_TEST', '80_system/Attach/11_alx/40_バグ管理');

INSERT INTO aws_storage (code,name,subscribe,is_upload,is_download,link_available,name_alias,exclude_subscribe,file_only) VALUES
	 ('011', '01_起票済（エネコム確認）',        'to_エネコム',              true,false,   '{}','01_起票済',           '{to_エネコム,to_アレクシード（翻訳前）}',  false),
	 ('012', '01_起票済（エネコム確認）',        'to_アレクシード（翻訳前）', false,true,   '{}','01_起票済（翻訳前）',  '{to_エネコム,to_アレクシード（翻訳前）}',  true),
	 ('02',  '02_原因確認中（アレクシード確認）', 'to_アレクシード',          false,true,   '{03}','02_原因確認中',     '{to_アレクシード}',                       false),
	 ('03',  '03_対応確認中（エネコム確認）',    'to_エネコム',              true,false,   '{03}','03_対応確認中',     '{to_エネコム}',                           false),
	 ('04',  '04_対応中（アレクシード確認）',    'to_アレクシード',          false,true,   '{03,05}','04_対応中',      '{to_アレクシード}',                       false),
	 ('05',  '05_対応済（アレクシード確認）',     'to_エネコム',             true,false,   '{}','05_対応済',           '{to_エネコム}',                          false),
	 ('06',  '06_完了（エネコム確認）',          'to_完了',                 false,false,  '{}','06_完了',             '{to_完了}',                              false);
