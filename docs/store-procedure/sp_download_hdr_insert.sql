-- ============================================================================
-- sp_download_hdr_insert
-- Insert a download header record and return the generated id.
-- ============================================================================

CREATE OR REPLACE FUNCTION sp_download_hdr_insert(
    p_download_ymd  VARCHAR(8),
    p_download_hms  VARCHAR(6),
    p_aws_cd        VARCHAR(3),
    p_sync_path     VARCHAR(255),
    p_download_count INTEGER,
    p_created_by    VARCHAR(100)
)
RETURNS TABLE (
    id INTEGER
)
LANGUAGE plpgsql
AS $$
BEGIN
    RETURN QUERY
    INSERT INTO download_hdr (download_ymd, download_hms, aws_cd, sync_path, download_count, created_by)
    VALUES (p_download_ymd, p_download_hms, p_aws_cd, p_sync_path, p_download_count, p_created_by)
    RETURNING download_hdr.id;
END;
$$;
