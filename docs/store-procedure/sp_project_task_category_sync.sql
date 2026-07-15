-- ============================================================================
-- sp_project_task_category_sync
-- Replace all categories for a task: delete existing, insert new list.
-- Frontend gửi process_code (VARCHAR), SP tự lookup sang categories.id.
-- ============================================================================
DROP FUNCTION IF EXISTS sp_project_task_category_sync(VARCHAR, VARCHAR[]);

CREATE OR REPLACE FUNCTION sp_project_task_category_sync(
    p_task_id         VARCHAR(50),
    p_process_codes   VARCHAR[]
)
RETURNS VOID
LANGUAGE plpgsql
AS $$
BEGIN
    DELETE FROM project_task_categories WHERE task_id = p_task_id;

    INSERT INTO project_task_categories (task_id, category_id)
    SELECT p_task_id, cat.id
    FROM unnest(p_process_codes) AS pc(code)
    JOIN categories cat ON cat.process_code = pc.code;
END;
$$;
