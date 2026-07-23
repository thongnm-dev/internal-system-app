-- ============================================================================
-- sp_ai_task_insert
-- Insert a new AI task. Returns the created task.
-- ============================================================================

CREATE OR REPLACE FUNCTION sp_ai_task_insert(
    p_task_code   VARCHAR(100),
    p_category    VARCHAR(30),
    p_description TEXT,
    p_created_by  VARCHAR(100)
)
RETURNS TABLE (
    id          INTEGER,
    task_code   VARCHAR(100),
    category    VARCHAR(30),
    description TEXT,
    created_by  VARCHAR(100),
    created_at  TEXT
)
LANGUAGE plpgsql
AS $$
BEGIN
    RETURN QUERY
    INSERT INTO ai_tasks (task_code, category, description, created_by)
    VALUES (p_task_code, p_category, p_description, p_created_by)
    RETURNING
        ai_tasks.id,
        ai_tasks.task_code,
        ai_tasks.category,
        ai_tasks.description,
        ai_tasks.created_by,
        ai_tasks.created_at::text;
END;
$$;
