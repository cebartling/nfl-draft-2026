-- Step 1: Add nullable column (no rewrite, fast)
ALTER TABLE drafts
    ADD COLUMN name VARCHAR(255);

-- Step 2: Backfill existing rows with a name based on their year
UPDATE drafts
    SET name = 'Draft - ' || TO_CHAR(created_at AT TIME ZONE 'UTC', 'Mon DD, YYYY HH12:MI AM')
    WHERE name IS NULL;

-- Step 3: Set NOT NULL constraint and static default for new rows
ALTER TABLE drafts
    ALTER COLUMN name SET NOT NULL,
    ALTER COLUMN name SET DEFAULT 'Untitled Draft';
