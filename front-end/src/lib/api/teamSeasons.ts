import { z } from 'zod';
import { apiClient } from './client';
import { TeamSeasonSchema, type TeamSeason } from '$lib/types';

/**
 * Team Seasons API module
 */
export const teamSeasonsApi = {
	/**
	 * List all team seasons for a given year
	 */
	async listByYear(year: number): Promise<TeamSeason[]> {
		return apiClient.get(`/team-seasons?year=${year}`, z.array(TeamSeasonSchema));
	},

	/**
	 * Get a specific team's season for a given year
	 */
	async getByTeamAndYear(teamId: string, year: number): Promise<TeamSeason | null> {
		try {
			return await apiClient.get(`/teams/${teamId}/seasons/${year}`, TeamSeasonSchema);
		} catch (error) {
			if (error instanceof Error && 'status' in error && error.status === 404) {
				return null;
			}
			throw error;
		}
	},
};
