-- Drop redundant index: idx_prospect_rankings_rank on (ranking_source_id, rank)
-- already covers source-only lookups since ranking_source_id is its leading column.
DROP INDEX IF EXISTS idx_prospect_rankings_source;
