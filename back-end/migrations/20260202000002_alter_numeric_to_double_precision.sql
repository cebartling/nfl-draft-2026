-- Alter combine_results NUMERIC columns to DOUBLE PRECISION for SQLx compatibility
ALTER TABLE combine_results
    ALTER COLUMN forty_yard_dash TYPE DOUBLE PRECISION,
    ALTER COLUMN vertical_jump TYPE DOUBLE PRECISION,
    ALTER COLUMN three_cone_drill TYPE DOUBLE PRECISION,
    ALTER COLUMN twenty_yard_shuttle TYPE DOUBLE PRECISION;

-- Alter scouting_reports grade column to DOUBLE PRECISION
ALTER TABLE scouting_reports
    ALTER COLUMN grade TYPE DOUBLE PRECISION;

-- Update constraints for combine_results
ALTER TABLE combine_results
    DROP CONSTRAINT valid_forty_dash,
    DROP CONSTRAINT valid_vertical,
    DROP CONSTRAINT valid_three_cone,
    DROP CONSTRAINT valid_shuttle,
    ADD CONSTRAINT valid_forty_dash CHECK (forty_yard_dash IS NULL OR (forty_yard_dash >= 4.0 AND forty_yard_dash <= 6.0)),
    ADD CONSTRAINT valid_vertical CHECK (vertical_jump IS NULL OR (vertical_jump >= 20 AND vertical_jump <= 50)),
    ADD CONSTRAINT valid_three_cone CHECK (three_cone_drill IS NULL OR (three_cone_drill >= 6.0 AND three_cone_drill <= 9.0)),
    ADD CONSTRAINT valid_shuttle CHECK (twenty_yard_shuttle IS NULL OR (twenty_yard_shuttle >= 3.5 AND twenty_yard_shuttle <= 6.0));

-- Update constraint for scouting_reports
ALTER TABLE scouting_reports
    DROP CONSTRAINT valid_grade,
    ADD CONSTRAINT valid_grade CHECK (grade >= 0.0 AND grade <= 10.0);
