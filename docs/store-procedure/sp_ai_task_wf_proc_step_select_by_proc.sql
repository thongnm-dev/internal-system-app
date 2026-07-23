-- ============================================================================
-- sp_ai_task_wf_proc_step_select_by_proc
-- List all step records for a workflow process.
-- ============================================================================

CREATE OR REPLACE FUNCTION sp_ai_task_wf_proc_step_select_by_proc(
    p_wf_proc_id INTEGER
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
    SELECT
        s.id,
        s.wf_proc_id,
        s.wf_step_id,
        s.status,
        s.created_at::text,
        s.created_by,
        s.updated_at::text,
        s.updated_by
    FROM ai_task_wf_proc_step s
    WHERE s.wf_proc_id = p_wf_proc_id
    ORDER BY s.created_at ASC;
END;
$$;
