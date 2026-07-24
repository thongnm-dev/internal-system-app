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
INSERT INTO users (username, password_hash, full_name, position, staff_no)
VALUES (
    'Thongnm',
    '$2b$12$dTyhgIqskYwXMkSfe6Luyuq0Ve7EMFS7Rrq7Z5eXvx7apv0bk9cOy',
    'Nguyen Minh Thong',
    'Admin',
    '10137'
)
ON CONFLICT (username) DO NOTHING;

-- Gán role admin cho user admin
INSERT INTO user_roles (user_id, role_id)
SELECT u.id, r.id
FROM users u, roles r
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
    ('copyTools',       'Copy Tools',        '/copy-tools',        'pi-copy',       'Tools',      TRUE, 6),
    ('checkMonthlyReport', 'Check Monthly Report', '/check-monthly-report', 'pi-database', 'Tools', TRUE, 7),
    ('sqlEditor',       'SQL Editor',        '/sql-editor',        'pi-server',     'Tools',      TRUE, 8),
    ('exploreFaster',   'Explore Faster',    '/explore-faster',    'pi-compass',    'Tools',      TRUE, 9),
    ('cloudS3',         'S3 Browser',        '/cloud/s3',          'pi-folder-open','Cloud',      TRUE, 10),
    ('cloudS3Upload',   'S3 Upload',         '/cloud/s3-upload',   'pi-upload',     'Cloud',      TRUE, 11),
    ('cloudS3Download', 'S3 Download',       '/cloud/s3-download', 'pi-download',   'Cloud',      TRUE, 12),
    ('cloudS3BugFolders', 'S3 Bug Folders', '/cloud/s3-bug-folders', 'pi-list', 'Cloud', TRUE, 13),
    ('cloudS3UploadHistory', 'S3 Upload History', '/cloud/s3-upload-history', 'pi-check-square', 'Cloud', true, 14),
    ('cloudS3DownloadHistory', 'S3 Download History', '/cloud/s3-download-history', 'pi-history', 'Cloud', true, 15),
    ('aiChat',          'AI Chat',           '/ai/chat',           'pi-comments',   'AI Agent',   TRUE, 16),
    ('aiUsage',         'AI Usage',          '/ai/usage',          'pi-chart-bar',  'AI Agent',   TRUE, 17),
    ('aiWorkflow',      'AI Workflow',       '/ai/workflow',       'pi-sitemap',    'AI Agent',   TRUE, 18),
    ('aiCowork',        'AI Cowork',         '/ai/cowork',         'pi-objects-column', 'AI Agent', TRUE, 19),
    ('aiTranslateCowork', 'AI Translate Cowork', '/ai/translate-cowork', 'pi-language', 'AI Agent', TRUE, 30),
    ('aiTasks',         'AI Tasks',          '/ai/tasks',          'pi-check-square', 'AI Agent',  TRUE, 20),
    ('projectSkills',   'Skills',            '/project-skills',    'pi-book',       'Governance', TRUE, 21),
    ('governanceMenus', 'Menus',             '/governance/menus',  'pi-bars',       'Governance', TRUE, 22),
    ('governanceUsers', 'Users',             '/governance/users',  'pi-users',      'Governance', TRUE, 23),
    ('governanceRoles', 'Roles',             '/governance/roles',  'pi-shield',     'Governance', TRUE, 24),
    ('governancePermissions', 'Permissions',  '/governance/permissions', 'pi-lock', 'Governance', TRUE, 25),
    ('governanceLogs',  'Logs',              '/governance/logs',   'pi-history',    'Governance', TRUE, 26),
    ('governanceAppConfig', 'App Config',    '/governance/app-config', 'pi-sliders-h', 'Governance', TRUE, 27),
    ('governanceStoreProcedure', 'Store Procedure', '/governance/store-procedure', 'pi-database', 'Governance', TRUE, 28),
    ('settings',        'Settings',          '/settings',          'pi-cog',        '—',          TRUE, 29)
ON CONFLICT (key) DO NOTHING;

-- Cấp toàn bộ menu cho role admin. Bắt buộc: sidebar/route giờ lấy hoàn toàn
-- từ menu_configs + quyền hiệu lực, mặc định mọi quyền là FALSE, nên nếu không
-- seed dòng này thì admin đăng nhập vào sẽ thấy sidebar rỗng.
INSERT INTO role_menu_permissions (role_id, menu_key)
SELECT r.id, m.key
FROM roles r
CROSS JOIN menu_configs m
WHERE r.name = 'admin'
ON CONFLICT (role_id, menu_key) DO NOTHING;

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

-- Project mặc định
INSERT INTO public.projects (id, code, "name", client, project_backlog_key, project_backlog_code, project_backlog_name, is_active) VALUES
    (1, '00000785', '【エネコム】41_ESS_TT_Test (総合テスト)', 'ENERCOM', 'EC41_ESS_IT_TEST', 661712, 'ESS_結合テスト', true)
ON CONFLICT (id) DO NOTHING;

-- Project member mặc định
INSERT INTO public.project_members (id, project_id, username, "name") VALUES(1, 1, 'Thongnm', 'Thongnm')
ON CONFLICT (id) DO NOTHING;