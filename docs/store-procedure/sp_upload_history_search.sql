-- ============================================================================
-- sp_upload_history_search
-- Search upload history with filters.
-- ============================================================================

CREATE OR REPLACE FUNCTION sp_upload_history_search(
    p_from_date      VARCHAR(8),
    p_to_date        VARCHAR(8),
    p_aws_cd         VARCHAR(3),
    p_bug_no         VARCHAR(100),
    p_is_moved_at_s3 BOOLEAN
)
RETURNS TABLE (
    upload_ymd    VARCHAR(8),
    aws_cd        VARCHAR(3),
    aws_name      VARCHAR(100),
    is_moved_at_s3 BOOLEAN,
    bug_no        VARCHAR(100),
    att_files     TEXT
)
LANGUAGE plpgsql
AS $$
BEGIN
    RETURN QUERY
    SELECT
          t1.upload_ymd
        , t1.aws_cd
        , t4.name                                      AS aws_name
        , COALESCE(t1.is_moved_at_s3, false)           AS is_moved_at_s3
        , t2.bug_no
        , string_agg(t3.file_name, ', ')               AS att_files
    FROM upload_hdr t1
    INNER JOIN upload_dtl t2
        ON t1.id = t2.upload_id
    INNER JOIN upload_attach t3
        ON t1.id = t3.upload_id
        AND t2.id = t3.upload_dtl_id
    INNER JOIN aws_storage t4
        ON t1.aws_cd = t4.code
    WHERE 1 = 1
        AND (
            (TRIM(p_from_date) = '' AND TRIM(p_to_date) <> '' AND t1.upload_ymd <= p_to_date)
            OR (TRIM(p_from_date) <> '' AND TRIM(p_to_date) = '' AND t1.upload_ymd >= p_from_date)
            OR (TRIM(p_from_date) <> '' AND TRIM(p_to_date) <> '' AND t1.upload_ymd BETWEEN p_from_date AND p_to_date)
            OR (TRIM(p_from_date) = '' AND TRIM(p_to_date) = '')
        )
        AND (TRIM(p_aws_cd) = '' OR t1.aws_cd = p_aws_cd)
        AND (TRIM(p_bug_no) = '' OR t2.bug_no LIKE '%' || p_bug_no || '%')
        AND (p_is_moved_at_s3 = FALSE OR COALESCE(t1.is_moved_at_s3, false) = p_is_moved_at_s3)
    GROUP BY
          t1.upload_ymd
        , t1.aws_cd
        , t4.name
        , t1.is_moved_at_s3
        , t2.bug_no
    ORDER BY
          t1.upload_ymd DESC
        , t1.aws_cd
        , t2.bug_no;
END;
$$;
