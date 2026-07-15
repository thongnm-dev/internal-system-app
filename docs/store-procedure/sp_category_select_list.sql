-- ============================================================================
-- sp_category_select_list
-- Danh sách công đoạn (process/phase) từ bảng categories, theo display_order.
-- ============================================================================
DROP FUNCTION IF EXISTS sp_category_select_list();

CREATE OR REPLACE FUNCTION sp_category_select_list()
RETURNS TABLE (
    id            INTEGER,
    process_code  VARCHAR(50),
    process_name  VARCHAR(200),
    short_name    VARCHAR(200),
    display_order INTEGER
)
LANGUAGE plpgsql
AS $$
BEGIN
    RETURN QUERY
    SELECT c.id, c.process_code, c.process_name, c.short_name, c.display_order
    FROM categories c
    ORDER BY c.display_order, c.process_code;
END;
$$;
