-- ============================================================================
-- sp_aws_storage_select_by_codes
-- List AWS storage configs whose link_available contains the given code.
-- Used to find delete options after upload.
-- ============================================================================

CREATE OR REPLACE FUNCTION sp_aws_storage_select_by_codes(
    p_code TEXT
)
RETURNS TABLE (
    id                INTEGER,
    code              VARCHAR(3),
    name              VARCHAR(100),
    name_alias        VARCHAR(100),
    subscribe         VARCHAR(100),
    is_upload         BOOLEAN,
    is_download       BOOLEAN,
    file_only         BOOLEAN,
    link_available    TEXT[],
    exclude_subscribe TEXT[]
)
LANGUAGE plpgsql
AS $$
BEGIN
    RETURN QUERY
    SELECT
        s.id,
        s.code,
        s.name,
        s.name_alias,
        s.subscribe,
        s.is_upload,
        s.is_download,
        s.file_only,
        s.link_available,
        s.exclude_subscribe
    FROM aws_storage s
    WHERE s.link_available @> ARRAY[p_code]
    ORDER BY s.code;
END;
$$;
