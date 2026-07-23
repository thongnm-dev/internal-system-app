-- ============================================================================
-- sp_ai_workflow_step_insert
-- Insert a new step into a workflow. Returns the created step.
-- ============================================================================

DROP FUNCTION IF EXISTS sp_ai_workflow_step_insert(INTEGER, VARCHAR, VARCHAR, TEXT, VARCHAR, INTEGER);

CREATE OR REPLACE FUNCTION sp_ai_workflow_step_insert(
    p_workflow_id INTEGER,
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
    INSERT INTO ai_workflow_steps (workflow_id, name, step_type, skill_name, description, icon, step_order)
    VALUES (p_workflow_id, p_name, p_step_type, p_skill_name, p_description, p_icon, p_step_order)
    RETURNING
        ai_workflow_steps.id,
        ai_workflow_steps.workflow_id,
        ai_workflow_steps.name,
        ai_workflow_steps.step_type,
        ai_workflow_steps.skill_name,
        ai_workflow_steps.description,
        ai_workflow_steps.icon,
        ai_workflow_steps.step_order,
        ai_workflow_steps.created_at::text;
END;
$$;
