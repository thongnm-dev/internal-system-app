-- ============================================================================
-- sp_project_task_category_sync
-- Replace all categories for a task: delete existing, insert new list.
-- ============================================================================

CREATE OR REPLACE FUNCTION sp_project_task_category_sync(
    p_task_id    VARCHAR(50),
    p_categories VARCHAR(30)[]
)
RETURNS VOID
LANGUAGE plpgsql
AS $$
BEGIN
    DELETE FROM project_task_categories WHERE task_id = p_task_id;

    INSERT INTO project_task_categories (task_id, category)
    SELECT p_task_id, unnest(p_categories);
END;
$$;
