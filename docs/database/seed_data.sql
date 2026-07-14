-- ============================================================================
-- seed_data.sql
-- Dữ liệu mặc định khi khởi tạo database.
-- Idempotent: dùng ON CONFLICT DO NOTHING, chạy lại nhiều lần không lỗi.
-- ============================================================================

-- Role mặc định
INSERT INTO roles (name, description)
VALUES ('admin', 'Administrator with full access')
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
