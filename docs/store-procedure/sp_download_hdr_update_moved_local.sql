-- ============================================================================
-- sp_download_hdr_update_moved_local
-- Mark a download_hdr record as moved to local and store the copied path.
-- ============================================================================

CREATE OR REPLACE FUNCTION sp_download_hdr_update_moved_local(
    p_id          INTEGER,
    p_path_copied VARCHAR(255)
)
RETURNS VOID
LANGUAGE plpgsql
AS $$
BEGIN
    UPDATE download_hdr
    SET is_moved_at_local = TRUE,
        updated_at        = CURRENT_TIMESTAMP
    WHERE id = p_id;

    UPDATE download_dtl
    SET path_copied = p_path_copied,
        updated_at  = CURRENT_TIMESTAMP
    WHERE download_id = p_id;
END;
$$;
