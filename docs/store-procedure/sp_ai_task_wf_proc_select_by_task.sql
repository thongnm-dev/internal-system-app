-- ============================================================================
-- sp_ai_task_wf_proc_select_by_task
-- List all workflow processes for a given task.
-- ============================================================================

CREATE OR REPLACE FUNCTION sp_ai_task_wf_proc_select_by_task(
    p_task_id INTEGER
)
RETURNS TABLE (
    id             INTEGER,
    task_id        INTEGER,
    wf_id          INTEGER,
    latest_step_id INTEGER,
    created_at     TEXT,
    created_by     VARCHAR(100),
    updated_at     TEXT,
    updated_by     VARCHAR(100)
)
LANGUAGE plpgsql
AS $$
BEGIN
    RETURN QUERY
    SELECT
        p.id,
        p.task_id,
        p.wf_id,
        p.latest_step_id,
        p.created_at::text,
        p.created_by,
        p.updated_at::text,
        p.updated_by
    FROM ai_task_wf_proc p
    WHERE p.task_id = p_task_id
    ORDER BY p.created_at DESC;
END;
$$;
