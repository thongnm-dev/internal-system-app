-- ============================================================================
-- sp_ai_workflow_step_update
-- Update a workflow step. Returns updated record.
-- ============================================================================

DROP FUNCTION IF EXISTS sp_ai_workflow_step_update(INTEGER, VARCHAR, VARCHAR, TEXT, VARCHAR, INTEGER);

CREATE OR REPLACE FUNCTION sp_ai_workflow_step_update(
    p_id          INTEGER,
    p_name        VARCHAR(200),
    p_step_type   VARCHAR(20),
    p_skill_name  VARCHAR(200),
    p_description TEXT,
    p_icon        VARCHAR(50),
    p_step_order  INTEGER
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
    UPDATE ai_workflow_steps s
    SET name = p_name,
        step_type = p_step_type,
        skill_name = p_skill_name,
        description = p_description,
        icon = p_icon,
        step_order = p_step_order
    WHERE s.id = p_id
    RETURNING
        s.id,
        s.workflow_id,
        s.name,
        s.step_type,
        s.skill_name,
        s.description,
        s.icon,
        s.step_order,
        s.created_at::text;
END;
$$;
