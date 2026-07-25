-- ============================================================================
-- sp_ai_model_select_list
-- List all AI models, ordered by provider then id.
-- ============================================================================

DROP FUNCTION IF EXISTS sp_ai_model_select_list();

CREATE OR REPLACE FUNCTION sp_ai_model_select_list()
RETURNS TABLE (
    id       INTEGER,
    provider VARCHAR(50),
    model    VARCHAR(100),
    version  VARCHAR(50)
)
LANGUAGE plpgsql
AS $$
BEGIN
    RETURN QUERY
    SELECT
        m.id,
        m.provider,
        m.model,
        m.version
    FROM ai_models m
    ORDER BY m.provider ASC, m.id ASC;
END;
$$;
