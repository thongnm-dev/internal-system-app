-- ============================================================================
-- sp_ai_workflow_step_reorder
-- Reorder all steps in a workflow by receiving a comma-separated list of step IDs.
-- Updates step_order based on position in the list.
-- ============================================================================

CREATE OR REPLACE FUNCTION sp_ai_workflow_step_reorder(
    p_workflow_id INTEGER,
    p_step_ids    INTEGER[]
)
RETURNS VOID
LANGUAGE plpgsql
AS $$
DECLARE
    v_order INTEGER := 0;
    v_id    INTEGER;
BEGIN
    FOREACH v_id IN ARRAY p_step_ids
    LOOP
        UPDATE ai_workflow_steps
        SET step_order = v_order
        WHERE id = v_id AND workflow_id = p_workflow_id;
        v_order := v_order + 1;
    END LOOP;

    UPDATE ai_workflows SET updated_at = NOW() WHERE id = p_workflow_id;
END;
$$;
