-- ============================================================================
-- sp_category_select_task_list
-- Danh sách category dùng cho project tasks (is_task_category = TRUE).
-- ============================================================================
DROP FUNCTION IF EXISTS sp_category_select_task_list();

CREATE OR REPLACE FUNCTION sp_category_select_task_list()
RETURNS TABLE (
    process_code  VARCHAR(50),
    process_name  VARCHAR(200),
    display_order INTEGER
)
LANGUAGE plpgsql
AS $$
BEGIN
    RETURN QUERY
    SELECT c.process_code, c.process_name, c.display_order
    FROM categories c
    WHERE c.is_task_category = TRUE
    ORDER BY c.display_order, c.process_code;
END;
$$;
