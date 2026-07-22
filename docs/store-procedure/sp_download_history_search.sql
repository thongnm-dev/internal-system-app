-- ============================================================================
-- sp_download_history_search
-- Search download history with filters.
-- ============================================================================

CREATE OR REPLACE FUNCTION sp_download_history_search(
    p_from_date         VARCHAR(8),
    p_to_date           VARCHAR(8),
    p_aws_cd            VARCHAR(3),
    p_bug_no            VARCHAR(100),
    p_is_moved_at_local BOOLEAN,
    p_is_moved_at_s3    BOOLEAN
)
RETURNS TABLE (
    id                INTEGER,
    download_ymd      VARCHAR(8),
    aws_cd            VARCHAR(3),
    aws_name          VARCHAR(100),
    is_moved_at_local BOOLEAN,
    bug_no            VARCHAR(100)
)
LANGUAGE plpgsql
AS $$
BEGIN
    RETURN QUERY
    SELECT
          t1.id
        , t1.download_ymd
        , t1.aws_cd
        , t3.name              AS aws_name
        , COALESCE(t1.is_moved_at_local, false) AS is_moved_at_local
        , t2.bug_no
    FROM download_hdr t1
    INNER JOIN download_dtl t2
        ON t1.id = t2.download_id
    INNER JOIN aws_storage t3
        ON t1.aws_cd = t3.code
    WHERE 1 = 1
        AND (
            (TRIM(p_from_date) = '' AND TRIM(p_to_date) <> '' AND t1.download_ymd <= p_to_date)
            OR (TRIM(p_from_date) <> '' AND TRIM(p_to_date) = '' AND t1.download_ymd >= p_from_date)
            OR (TRIM(p_from_date) <> '' AND TRIM(p_to_date) <> '' AND t1.download_ymd BETWEEN p_from_date AND p_to_date)
            OR (TRIM(p_from_date) = '' AND TRIM(p_to_date) = '')
        )
        AND (TRIM(p_aws_cd) = '' OR t1.aws_cd = p_aws_cd)
        AND (TRIM(p_bug_no) = '' OR t2.bug_no LIKE '%' || p_bug_no || '%')
        AND (p_is_moved_at_local = FALSE OR COALESCE(t1.is_moved_at_local, false) = p_is_moved_at_local)
        AND (p_is_moved_at_s3 = FALSE OR COALESCE(t2.is_moved_at_s3, false) = p_is_moved_at_s3)
    GROUP BY
          t1.id
        , t1.download_ymd
        , t1.aws_cd
        , t3.name
        , t1.is_moved_at_local
        , t2.bug_no
    ORDER BY
          t1.download_ymd DESC
        , t1.aws_cd
        , t2.bug_no;
END;
$$;
