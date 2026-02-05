/**
 * Utility functions for team logos
 */

/**
 * Get the path to a team's logo image
 * @param abbreviation - The team's abbreviation (e.g., 'KC', 'DAL')
 * @returns The path to the logo PNG file
 */
export function getTeamLogoPath(abbreviation: string): string {
	return `/logos/teams/${abbreviation}.png`;
}
