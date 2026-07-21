-- ============================================================================
-- sp_aws_storage_select_browser_allowed
-- Returns allowed S3 prefixes for the browser by joining aws_work_folder
-- with aws_storage where is_upload = true OR is_download = true.
-- ============================================================================

CREATE OR REPLACE FUNCTION sp_aws_storage_select_browser_allowed(
    p_folder_key VARCHAR(50)
)
RETURNS TABLE (
    allowed_prefix TEXT
)
LANGUAGE plpgsql
AS $$
BEGIN
    RETURN QUERY
    SELECT
        w.name || '/' || s.name || '/'
    FROM aws_storage s
    CROSS JOIN aws_work_folder w
    WHERE (s.is_upload = true OR s.is_download = true)
      AND w.folder_key = p_folder_key
    ORDER BY s.code;
END;
$$;
