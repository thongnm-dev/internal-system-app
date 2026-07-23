-- ============================================================================
-- sp_ai_task_wf_proc_insert
-- Insert a workflow process for a task. Returns the created row.
-- ============================================================================

CREATE OR REPLACE FUNCTION sp_ai_task_wf_proc_insert(
    p_task_id    INTEGER,
    p_wf_id      INTEGER,
    p_created_by VARCHAR(100)
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
    INSERT INTO ai_task_wf_proc (task_id, wf_id, created_by, updated_by)
    VALUES (p_task_id, p_wf_id, p_created_by, p_created_by)
    RETURNING
        ai_task_wf_proc.id,
        ai_task_wf_proc.task_id,
        ai_task_wf_proc.wf_id,
        ai_task_wf_proc.latest_step_id,
        ai_task_wf_proc.created_at::text,
        ai_task_wf_proc.created_by,
        ai_task_wf_proc.updated_at::text,
        ai_task_wf_proc.updated_by;
END;
$$;
