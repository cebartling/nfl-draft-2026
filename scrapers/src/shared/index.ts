export { normalizePosition } from "./position-normalizer.js";
export { cleanName, normalizeLastName, nameKey } from "./name-normalizer.js";
export {
  normalizeSvgAbbreviation,
  resolveTeamAbbreviation,
  VALID_TEAM_ABBREVIATIONS,
} from "./team-abbreviations.js";
export { writeJsonFile, isTemplateData, shouldPreventOverwrite } from "./json-writer.js";
