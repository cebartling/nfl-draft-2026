-- Fix new measurable columns to use DOUBLE PRECISION to match existing columns
-- SQLx maps NUMERIC to BigDecimal but DOUBLE PRECISION to f64

ALTER TABLE combine_results
    ALTER COLUMN arm_length TYPE DOUBLE PRECISION,
    ALTER COLUMN hand_size TYPE DOUBLE PRECISION,
    ALTER COLUMN wingspan TYPE DOUBLE PRECISION,
    ALTER COLUMN ten_yard_split TYPE DOUBLE PRECISION,
    ALTER COLUMN twenty_yard_split TYPE DOUBLE PRECISION;
