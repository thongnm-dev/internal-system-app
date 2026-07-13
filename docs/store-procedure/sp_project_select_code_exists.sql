-- ============================================================================
-- sp_project_select_code_exists
-- Check if a project code already exists, optionally excluding a project ID.
-- ============================================================================

CREATE OR REPLACE FUNCTION sp_project_select_code_exists(
    p_code       VARCHAR(20),
    p_exclude_id INTEGER DEFAULT NULL
)
RETURNS BOOLEAN
LANGUAGE plpgsql
AS $$
DECLARE
    v_exists BOOLEAN;
BEGIN
    IF p_exclude_id IS NOT NULL THEN
        SELECT EXISTS(
            SELECT 1 FROM projects
            WHERE LOWER(code) = LOWER(p_code) AND id != p_exclude_id
        ) INTO v_exists;
    ELSE
        SELECT EXISTS(
            SELECT 1 FROM projects
            WHERE LOWER(code) = LOWER(p_code)
        ) INTO v_exists;
    END IF;

    RETURN v_exists;
END;
$$;
