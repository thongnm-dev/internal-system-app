-- ============================================================================
-- sp_upload_dtl_insert
-- Insert a new upload detail record, returning the generated id.
-- ============================================================================

CREATE OR REPLACE FUNCTION sp_upload_dtl_insert(
    p_upload_id INTEGER,
    p_bug_no    VARCHAR(100)
)
RETURNS TABLE (id INTEGER)
LANGUAGE plpgsql
AS $$
BEGIN
    RETURN QUERY
    INSERT INTO upload_dtl (upload_id, bug_no)
    VALUES (p_upload_id, p_bug_no)
    RETURNING upload_dtl.id;
END;
$$;
