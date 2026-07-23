-- ============================================================================
-- sp_ai_task_wf_proc_update
-- Update the latest step of a workflow process. Returns the updated row.
-- ============================================================================

CREATE OR REPLACE FUNCTION sp_ai_task_wf_proc_update(
    p_id             INTEGER,
    p_latest_step_id INTEGER,
    p_updated_by     VARCHAR(100)
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
    UPDATE ai_task_wf_proc p
    SET latest_step_id = p_latest_step_id,
        updated_by     = p_updated_by
    WHERE p.id = p_id
    RETURNING
        p.id,
        p.task_id,
        p.wf_id,
        p.latest_step_id,
        p.created_at::text,
        p.created_by,
        p.updated_at::text,
        p.updated_by;
END;
$$;
