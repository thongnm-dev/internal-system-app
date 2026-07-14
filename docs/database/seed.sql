-- ============================================================================
-- PJ Yuji Internal Tool — Seed Data
-- Run after schema.sql to populate default records.
-- ============================================================================

-- Roles
INSERT INTO roles (name, description) VALUES
    ('admin',  'Full system access including governance screens'),
    ('member', 'Standard user with project and daily report access'),
    ('viewer', 'Read-only access to overview and reports');

-- Projects (from Daily Report assigned + optional projects)
-- INSERT INTO projects (code, name, client, backlog_key, backlog_url, backlog_space) VALUES
--     ('YUJI', 'PJ Yuji Internal Tool',   'Internal',   'INT',  '', ''),
--     ('HRP',  'HR Portal',               'Operations', 'HRP',  '', ''),
--     ('SALE', 'Sales Dashboard',          'Business',   'SALE', '', ''),
--     ('MOB',  'Mobile Companion App',     'Product',    'MOB',  '', ''),
--     ('OPS',  'Infrastructure Support',   'Platform',   'OPS',  '', '');

-- Default project tasks (from Daily Report screen)
-- INSERT INTO project_tasks (id, project_id, short_name) VALUES
--     ('yuji-planning', (SELECT id FROM projects WHERE code = 'YUJI'), 'Planning and daily coordination'),
--     ('yuji-frontend', (SELECT id FROM projects WHERE code = 'YUJI'), 'Frontend implementation'),
--     ('yuji-review',   (SELECT id FROM projects WHERE code = 'YUJI'), 'Code review and QA support'),
--     ('hrp-maintenance',(SELECT id FROM projects WHERE code = 'HRP'), 'Maintenance requests'),
--     ('hrp-bugfix',    (SELECT id FROM projects WHERE code = 'HRP'),  'Bug fixing'),
--     ('sale-reporting', (SELECT id FROM projects WHERE code = 'SALE'), 'Report screen updates'),
--     ('sale-data',     (SELECT id FROM projects WHERE code = 'SALE'),  'Data reconciliation'),
--     ('sale-meeting',  (SELECT id FROM projects WHERE code = 'SALE'),  'Stakeholder meeting'),
--     ('mob-api',       (SELECT id FROM projects WHERE code = 'MOB'),   'API integration'),
--     ('mob-test',      (SELECT id FROM projects WHERE code = 'MOB'),   'Device testing'),
--     ('ops-monitoring',(SELECT id FROM projects WHERE code = 'OPS'),   'Monitoring and alert handling'),
--     ('ops-release',   (SELECT id FROM projects WHERE code = 'OPS'),   'Release support');

-- Menu configuration (from Governance Menus screen)
INSERT INTO menu_configs (key, title, path, icon, menu_group, is_visible, display_order) VALUES
    ('overview',        'Overview',          '/overview',          'pi-home',       '—',          1, 0),
    ('projects',        'Projects',          '/projects',          'pi-table',      '—',          1, 1),
    ('issueBacklog',    'Issue Backlog',     '/issue-backlog',     'pi-list-check', '—',          1, 2),
    ('dailyWorkNotes',  'Daily Work Notes',  '/daily-work-notes',  'pi-pencil',     '—',          1, 3),
    ('dailyReport',     'Daily Report',      '/daily-report',      'pi-calendar',   '—',          1, 4),
    ('excel2md',        'Excel to Markdown', '/excel2md',          'pi-file',       'Tools',      1, 5),
    ('importCsv',       'Import CSV',        '/import-csv',        'pi-database',   'Tools',      1, 6),
    ('projectSkills',   'Skills',            '/project-skills',    'pi-book',       'Governance', 1, 7),
    ('governanceMenus', 'Menus',             '/governance/menus',  'pi-bars',       'Governance', 1, 8),
    ('governanceUsers', 'Users',             '/governance/users',  'pi-users',      'Governance', 1, 9),
    ('governanceLogs',  'Logs',              '/governance/logs',   'pi-history',    'Governance', 1, 10),
    ('settings',        'Settings',          '/settings',          'pi-cog',        '—',          1, 11);
