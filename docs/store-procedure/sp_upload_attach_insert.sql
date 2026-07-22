-- ============================================================================
-- sp_upload_attach_insert
-- Insert a new upload attachment record.
-- ============================================================================

CREATE OR REPLACE FUNCTION sp_upload_attach_insert(
    p_upload_id     INTEGER,
    p_upload_dtl_id INTEGER,
    p_file_name     VARCHAR(255),
    p_file_path     VARCHAR(255)
)
RETURNS VOID
LANGUAGE plpgsql
AS $$
BEGIN
    INSERT INTO upload_attach (upload_id, upload_dtl_id, file_name, file_path)
    VALUES (p_upload_id, p_upload_dtl_id, p_file_name, p_file_path);
END;
$$;
