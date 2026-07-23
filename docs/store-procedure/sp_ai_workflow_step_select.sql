-- ============================================================================
-- sp_ai_workflow_step_select
-- Select all steps for a workflow, ordered by step_order asc.
-- ============================================================================

DROP FUNCTION IF EXISTS sp_ai_workflow_step_select(INTEGER);

CREATE OR REPLACE FUNCTION sp_ai_workflow_step_select(
    p_workflow_id INTEGER
)
RETURNS TABLE (
    id          INTEGER,
    workflow_id INTEGER,
    name        VARCHAR(200),
    step_type   VARCHAR(20),
    skill_name  VARCHAR(200),
    description TEXT,
    icon        VARCHAR(50),
    step_order  INTEGER,
    created_at  TEXT
)
LANGUAGE plpgsql
AS $$
BEGIN
    RETURN QUERY
    SELECT
        s.id,
        s.workflow_id,
        s.name,
        s.step_type,
        s.skill_name,
        s.description,
        s.icon,
        s.step_order,
        s.created_at::text
    FROM ai_workflow_steps s
    WHERE s.workflow_id = p_workflow_id
    ORDER BY s.step_order ASC;
END;
$$;
