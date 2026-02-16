-- Add source column and additional measurables to combine_results
-- Supports both NFL Combine and Pro Day data sources

-- Add new columns
ALTER TABLE combine_results
    ADD COLUMN source VARCHAR(20) NOT NULL DEFAULT 'combine',
    ADD COLUMN arm_length DECIMAL(4,2),
    ADD COLUMN hand_size DECIMAL(4,2),
    ADD COLUMN wingspan DECIMAL(5,2),
    ADD COLUMN ten_yard_split DECIMAL(4,2),
    ADD COLUMN twenty_yard_split DECIMAL(4,2);

-- Add CHECK constraints for new columns
ALTER TABLE combine_results
    ADD CONSTRAINT valid_source CHECK (source IN ('combine', 'pro_day')),
    ADD CONSTRAINT valid_arm_length CHECK (arm_length IS NULL OR (arm_length >= 28.00 AND arm_length <= 40.00)),
    ADD CONSTRAINT valid_hand_size CHECK (hand_size IS NULL OR (hand_size >= 7.00 AND hand_size <= 12.00)),
    ADD CONSTRAINT valid_wingspan CHECK (wingspan IS NULL OR (wingspan >= 70.00 AND wingspan <= 90.00)),
    ADD CONSTRAINT valid_ten_yard CHECK (ten_yard_split IS NULL OR (ten_yard_split >= 1.30 AND ten_yard_split <= 2.10)),
    ADD CONSTRAINT valid_twenty_yard CHECK (twenty_yard_split IS NULL OR (twenty_yard_split >= 2.30 AND twenty_yard_split <= 3.50));

-- Drop old unique constraint and add new one that includes source
ALTER TABLE combine_results DROP CONSTRAINT unique_player_combine;
ALTER TABLE combine_results ADD CONSTRAINT unique_player_combine_source UNIQUE (player_id, year, source);

-- Add index on source for filtering
CREATE INDEX idx_combine_source ON combine_results(source);

COMMENT ON COLUMN combine_results.source IS 'Data source: combine (NFL Combine) or pro_day (Pro Day workout)';
COMMENT ON COLUMN combine_results.arm_length IS 'Arm length in inches';
COMMENT ON COLUMN combine_results.hand_size IS 'Hand size in inches';
COMMENT ON COLUMN combine_results.wingspan IS 'Wingspan in inches';
COMMENT ON COLUMN combine_results.ten_yard_split IS '10-yard split time in seconds';
COMMENT ON COLUMN combine_results.twenty_yard_split IS '20-yard split time in seconds';
