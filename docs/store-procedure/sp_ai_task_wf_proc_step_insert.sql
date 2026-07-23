-- ============================================================================
-- sp_ai_task_wf_proc_step_insert
-- Insert a step record for a workflow process. Returns the created row.
-- ============================================================================

CREATE OR REPLACE FUNCTION sp_ai_task_wf_proc_step_insert(
    p_wf_proc_id INTEGER,
    p_wf_step_id INTEGER,
    p_status     VARCHAR(20),
    p_created_by VARCHAR(100)
)
RETURNS TABLE (
    id         INTEGER,
    wf_proc_id INTEGER,
    wf_step_id INTEGER,
    status     VARCHAR(20),
    created_at TEXT,
    created_by VARCHAR(100),
    updated_at TEXT,
    updated_by VARCHAR(100)
)
LANGUAGE plpgsql
AS $$
BEGIN
    RETURN QUERY
    INSERT INTO ai_task_wf_proc_step (wf_proc_id, wf_step_id, status, created_by, updated_by)
    VALUES (p_wf_proc_id, p_wf_step_id, p_status, p_created_by, p_created_by)
    RETURNING
        ai_task_wf_proc_step.id,
        ai_task_wf_proc_step.wf_proc_id,
        ai_task_wf_proc_step.wf_step_id,
        ai_task_wf_proc_step.status,
        ai_task_wf_proc_step.created_at::text,
        ai_task_wf_proc_step.created_by,
        ai_task_wf_proc_step.updated_at::text,
        ai_task_wf_proc_step.updated_by;
END;
$$;
