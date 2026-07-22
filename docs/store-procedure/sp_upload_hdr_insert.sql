-- ============================================================================
-- sp_upload_hdr_insert
-- Insert a new upload header record, returning the generated id.
-- ============================================================================

CREATE OR REPLACE FUNCTION sp_upload_hdr_insert(
    p_upload_ymd    VARCHAR(8),
    p_upload_hms    VARCHAR(6),
    p_aws_cd        VARCHAR(3),
    p_upload_count  INTEGER,
    p_created_by    VARCHAR(100),
    p_is_moved_at_s3 BOOLEAN
)
RETURNS TABLE (id INTEGER)
LANGUAGE plpgsql
AS $$
BEGIN
    RETURN QUERY
    INSERT INTO upload_hdr (upload_ymd, upload_hms, aws_cd, upload_count, created_by, is_moved_at_s3)
    VALUES (p_upload_ymd, p_upload_hms, p_aws_cd, p_upload_count, p_created_by, p_is_moved_at_s3)
    RETURNING upload_hdr.id;
END;
$$;
