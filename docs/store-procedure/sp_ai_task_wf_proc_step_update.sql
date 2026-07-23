-- ============================================================================
-- sp_ai_task_wf_proc_step_update
-- Update the status of a workflow process step. Returns the updated row.
-- ============================================================================

CREATE OR REPLACE FUNCTION sp_ai_task_wf_proc_step_update(
    p_id         INTEGER,
    p_status     VARCHAR(20),
    p_updated_by VARCHAR(100)
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
    UPDATE ai_task_wf_proc_step s
    SET status     = p_status,
        updated_by = p_updated_by
    WHERE s.id = p_id
    RETURNING
        s.id,
        s.wf_proc_id,
        s.wf_step_id,
        s.status,
        s.created_at::text,
        s.created_by,
        s.updated_at::text,
        s.updated_by;
END;
$$;
