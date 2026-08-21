-- public-enrolment-totals: public weekly enrolment totals.
-- Params: {{ window_from }}, {{ window_to }}
SELECT
    DATE_TRUNC('week', enrolled_at)     AS week_start,
    county_code,
    COUNT(*)                            AS enrolments
FROM eel_enrolments
WHERE enrolled_at >= '{{ window_from }}'::timestamptz
  AND enrolled_at <  '{{ window_to }}'::timestamptz
  AND status = 'complete'
GROUP BY DATE_TRUNC('week', enrolled_at), county_code
ORDER BY week_start, county_code;
