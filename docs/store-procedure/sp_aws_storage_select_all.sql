-- ============================================================================
-- sp_aws_storage_select_all
-- List all AWS storage configs.
-- ============================================================================

CREATE OR REPLACE FUNCTION sp_aws_storage_select_all()
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
    ORDER BY s.code;
END;
$$;
