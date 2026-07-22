-- ============================================================================
-- sp_download_dtl_update_moved_s3
-- Mark download_dtl records as moved on S3 by aws_cd and bug_no list.
-- ============================================================================

CREATE OR REPLACE FUNCTION sp_download_dtl_update_moved_s3(
    p_aws_cd    VARCHAR(3),
    p_bug_nos   TEXT[]
)
RETURNS VOID
LANGUAGE plpgsql
AS $$
BEGIN
    UPDATE download_dtl
    SET is_moved_at_s3 = TRUE,
        updated_at     = CURRENT_TIMESTAMP
    WHERE download_id IN (
        SELECT id FROM download_hdr WHERE aws_cd = p_aws_cd
    )
    AND bug_no = ANY(p_bug_nos);
END;
$$;
