-- Allow multiple drafts per year
ALTER TABLE drafts DROP CONSTRAINT unique_draft_year;
