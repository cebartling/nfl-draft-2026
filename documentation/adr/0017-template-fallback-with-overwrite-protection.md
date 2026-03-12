# ADR 0017: Template Fallback with Overwrite Protection

## Status

Accepted

## Context

The data scraping pipeline fetches real-world data from external websites (PFR, Tankathon, Mockdraftable). These sites can be unavailable, change their HTML structure, or block automated requests (403 errors). When scraping fails, the pipeline needs a fallback to keep the development workflow functional.

Early in development, scraper failures silently produced template (synthetic) data and wrote it to the output file. This caused a subtle problem: a developer who had previously scraped real data could accidentally overwrite it with template data by re-running the scraper during a site outage. The real data would be lost with no warning.

### Forces

- **Development velocity**: Developers need working data even when external sources are down
- **Data preservation**: Real scraped data is time-consuming to obtain and should not be lost
- **Explicit failures**: Silent data degradation is worse than a visible error
- **CI compatibility**: Automated pipelines should not fail because an external site is temporarily down

## Decision

We implement a **two-layer safety system** for the data pipeline:

### 1. Template fallback on scrape failure

When a scraper fails (network error, 403, HTML parse failure), it falls back to generating template data — synthetic but structurally valid data that allows the rest of the pipeline to function. Template data is tagged with `meta.source` values like `"template"`, `"mock_generator"`, or `"generated"`.

### 2. Overwrite protection for curated data

Before writing output, the pipeline checks whether the destination file already contains non-template ("curated") data. If the new data is template-sourced and the existing file is curated, the write is **blocked** with an explicit error:

```typescript
// scrapers/src/shared/json-writer.ts
export function shouldPreventOverwrite(
  outputPath: string,
  newData: { meta: { source: string } }
): boolean {
  if (!existsSync(outputPath)) return false;

  const existing = JSON.parse(readFileSync(outputPath, "utf-8"));
  const existingSource = existing?.meta?.source ?? "";
  const newSource = newData.meta.source;

  const templateSources = ["template", "mock_generator", "generated"];
  const existingIsCurated = !templateSources.includes(existingSource);
  const newIsTemplate = templateSources.includes(newSource);

  return existingIsCurated && newIsTemplate;
}
```

The error message explains what happened and offers `--allow-template-fallback` to force the overwrite if intended.

## Consequences

### Positive

- **Data safety**: Real data is never silently replaced by synthetic data
- **Clear error messages**: When scraping fails and overwrite is blocked, the developer knows exactly what happened and why
- **Escape hatch**: `--allow-template-fallback` allows intentional overwrites without removing the protection
- **Development works offline**: Template data provides structurally valid data for local development when sources are unavailable

### Negative

- **Template data is misleading**: Template data uses realistic-looking but fake measurements. Developers must check `meta.source` to know whether they're working with real data
- **Stale data risk**: The protection prevents accidental overwrites but also means scraped data can become stale if the developer doesn't re-scrape when sources are available
- **Source detection is fragile**: The list of template source strings is hardcoded and must be kept in sync across the codebase

### Neutral

- **No automatic retries**: The pipeline does not retry failed scrapes. This keeps the pipeline fast and deterministic — retries are left to the developer or CI configuration
- **Per-file protection**: Each output file is checked independently. A merge pipeline might succeed for one source and fail for another

## Alternatives Considered

### Fail hard on scrape failure (no template fallback)

**Pros**: Forces developers to fix scraping issues immediately

**Cons**: Blocks all development when an external site is down. PFR in particular blocks automated requests intermittently

**Rejected**: External site availability should not gate local development

### Timestamped backup before overwrite

**Pros**: Preserves previous data automatically

**Cons**: Adds file management complexity. Developers still might not notice the data was replaced until they see unexpected values in the UI

**Rejected**: Prevention is better than recovery for this problem

### Git-based protection (check if file is modified)

**Pros**: Leverages existing version control

**Cons**: Data files are tracked in git, so they always appear "clean" after commit. Would only protect uncommitted changes

**Rejected**: Too narrow — the common case is overwriting already-committed curated data

## References

- `scrapers/src/shared/json-writer.ts` — `shouldPreventOverwrite()` implementation
- `scrapers/src/commands/combine.ts` — safety guard usage in combine command
- `scrapers/src/commands/rankings.ts` — safety guard usage in rankings command
