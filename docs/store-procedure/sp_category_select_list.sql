-- ============================================================================
-- sp_category_select_list
-- Danh sách công đoạn (process/phase) từ bảng categories, theo display_order.
-- process_code chứa luôn tên phase (PG, UT, Review PG…). Dùng cho dropdown Phase.
-- ============================================================================

-- Return type đổi (bỏ phase_name) -> DROP bản cũ trước.
DROP FUNCTION IF EXISTS sp_category_select_list();

CREATE OR REPLACE FUNCTION sp_category_select_list()
RETURNS TABLE (
    process_code  VARCHAR(50),
    display_order INTEGER
)
LANGUAGE plpgsql
AS $$
BEGIN
    RETURN QUERY
    SELECT c.process_code, c.display_order
    FROM categories c
    ORDER BY c.display_order, c.process_code;
END;
$$;
