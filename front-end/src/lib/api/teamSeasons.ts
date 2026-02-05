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
		const seasons = await this.listByYear(year);
		return seasons.find((s) => s.team_id === teamId) ?? null;
	},
};
