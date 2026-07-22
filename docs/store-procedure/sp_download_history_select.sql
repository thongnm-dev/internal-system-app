-- ============================================================================
-- sp_download_history_select
-- Get recent download history for a user, joined with aws_storage.
-- ============================================================================

CREATE OR REPLACE FUNCTION sp_download_history_select(
    p_created_by VARCHAR(100)
)
RETURNS TABLE (
    id                INTEGER,
    download_ymd      VARCHAR(8),
    download_hms      VARCHAR(6),
    sync_path         VARCHAR(255),
    download_count    INTEGER,
    is_moved_at_local BOOLEAN,
    aws_name          VARCHAR(100),
    aws_name_alias    VARCHAR(100)
)
LANGUAGE plpgsql
AS $$
BEGIN
    RETURN QUERY
    SELECT
        t1.id,
        t1.download_ymd,
        t1.download_hms,
        t1.sync_path,
        t1.download_count,
        COALESCE(t1.is_moved_at_local, false) AS is_moved_at_local,
        t3.name          AS aws_name,
        t3.name_alias    AS aws_name_alias
    FROM download_hdr t1
    INNER JOIN aws_storage t3 ON t1.aws_cd = t3.code
    WHERE t1.created_by = p_created_by
        AND t1.download_count > 0
        AND COALESCE(t1.is_moved_at_local, false) = false
    ORDER BY (t1.download_ymd || t1.download_hms) DESC
    LIMIT 50;
END;
$$;
