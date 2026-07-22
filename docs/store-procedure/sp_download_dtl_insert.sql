-- ============================================================================
-- sp_download_dtl_insert
-- Insert a download detail record.
-- ============================================================================

CREATE OR REPLACE FUNCTION sp_download_dtl_insert(
    p_download_id INTEGER,
    p_bug_no      VARCHAR(100),
    p_sync_path   VARCHAR(255)
)
RETURNS VOID
LANGUAGE plpgsql
AS $$
BEGIN
    INSERT INTO download_dtl (download_id, bug_no, sync_path)
    VALUES (p_download_id, p_bug_no, p_sync_path);
END;
$$;
