import { z } from 'zod';
import { apiClient } from './client';
import {
	TeamSchema,
	TeamNeedSchema,
	ScoutingReportSchema,
	type Team,
	type TeamNeed,
	type ScoutingReport,
} from '$lib/types';

/**
 * Teams API module
 */
export const teamsApi = {
	/**
	 * List all teams
	 */
	async list(): Promise<Team[]> {
		return apiClient.get('/teams', z.array(TeamSchema));
	},

	/**
	 * Get a single team by ID
	 */
	async get(id: string): Promise<Team> {
		return apiClient.get(`/teams/${id}`, TeamSchema);
	},

	/**
	 * Create a new team
	 */
	async create(team: Omit<Team, 'id'>): Promise<Team> {
		return apiClient.post('/teams', team, TeamSchema);
	},

	/**
	 * Update an existing team
	 */
	async update(id: string, team: Partial<Omit<Team, 'id'>>): Promise<Team> {
		return apiClient.put(`/teams/${id}`, team, TeamSchema);
	},

	/**
	 * Get team needs
	 */
	async getNeeds(teamId: string): Promise<TeamNeed[]> {
		return apiClient.get(`/teams/${teamId}/needs`, z.array(TeamNeedSchema));
	},

	/**
	 * Create a new team need
	 */
	async createNeed(need: Omit<TeamNeed, 'id'>): Promise<TeamNeed> {
		return apiClient.post('/teams/needs', need, TeamNeedSchema);
	},

	/**
	 * Get scouting reports for a team (sorted by grade DESC from backend)
	 */
	async getScoutingReports(teamId: string): Promise<ScoutingReport[]> {
		return apiClient.get(`/teams/${teamId}/scouting-reports`, z.array(ScoutingReportSchema));
	},
};
