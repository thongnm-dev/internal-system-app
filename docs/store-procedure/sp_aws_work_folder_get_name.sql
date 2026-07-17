-- ============================================================================
-- sp_aws_work_folder_get_name
-- Get the folder name by folder_key.
-- ============================================================================

CREATE OR REPLACE FUNCTION sp_aws_work_folder_get_name(
    p_folder_key VARCHAR(50)
)
RETURNS TABLE (
    name VARCHAR(100)
)
LANGUAGE plpgsql
AS $$
BEGIN
    RETURN QUERY
    SELECT w.name
    FROM aws_work_folder w
    WHERE w.folder_key = p_folder_key;
END;
$$;
