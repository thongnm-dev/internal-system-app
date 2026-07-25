-- ============================================================================
-- sp_ai_task_select_list
-- Search/list AI tasks by keyword (matches task_cd, task_name, category).
-- NULL/empty keyword returns the most recent tasks.
-- p_is_complete: NULL = all, TRUE = completed only, FALSE = incomplete only.
-- Also returns the workflow process/step currently in progress for each task
-- (the most recently updated ai_task_wf_proc_step with status 'in_progress').
-- ============================================================================

DROP FUNCTION IF EXISTS sp_ai_task_select_list(VARCHAR, BOOLEAN);

CREATE OR REPLACE FUNCTION sp_ai_task_select_list(
    p_keyword     VARCHAR(200) DEFAULT NULL,
    p_is_complete BOOLEAN      DEFAULT NULL
)
RETURNS TABLE (
    id                  INTEGER,
    task_cd             VARCHAR(100),
    task_name           VARCHAR(300),
    category            VARCHAR(30),
    is_complete         BOOLEAN,
    completed_at        TEXT,
    created_at          TEXT,
    created_by          VARCHAR(100),
    updated_at          TEXT,
    updated_by          VARCHAR(100),
    current_wf_name     TEXT,
    current_step_name   TEXT,
    current_step_status TEXT
)
LANGUAGE plpgsql
AS $$
BEGIN
    RETURN QUERY
    SELECT
        t.id,
        t.task_cd,
        t.task_name,
        t.category,
        t.is_complete,
        COALESCE(t.completed_at::text, ''),
        t.created_at::text,
        t.created_by,
        t.updated_at::text,
        t.updated_by,
        COALESCE(cur.wf_name, '')::text,
        COALESCE(cur.step_name, '')::text,
        COALESCE(cur.step_status, '')::text
    FROM ai_tasks t
    LEFT JOIN LATERAL (
        SELECT w.name AS wf_name, s.name AS step_name, ps.status AS step_status
        FROM ai_task_wf_proc p
        JOIN ai_task_wf_proc_step ps ON ps.wf_proc_id = p.id
        JOIN ai_workflows w ON w.id = p.wf_id
        JOIN ai_workflow_steps s ON s.id = ps.wf_step_id
        WHERE p.task_id = t.id AND ps.status = 'in_progress'
        ORDER BY ps.updated_at DESC
        LIMIT 1
    ) cur ON TRUE
    WHERE (p_keyword IS NULL OR p_keyword = ''
       OR t.task_cd ILIKE '%' || p_keyword || '%'
       OR t.task_name ILIKE '%' || p_keyword || '%'
       OR t.category ILIKE '%' || p_keyword || '%')
      AND (p_is_complete IS NULL OR t.is_complete = p_is_complete)
    ORDER BY t.created_at DESC
    LIMIT 200;
END;
$$;
