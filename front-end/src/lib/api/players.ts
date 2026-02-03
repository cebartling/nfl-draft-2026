import { z } from 'zod';
import { apiClient } from './client';
import {
	PlayerSchema,
	ScoutingReportSchema,
	CombineResultsSchema,
	type Player,
	type ScoutingReport,
	type CombineResults,
	type Position,
} from '$lib/types';

/**
 * Type guard for errors with HTTP status codes
 */
function hasStatus(error: unknown): error is Error & { status: number } {
	return error instanceof Error && 'status' in error && typeof (error as Error & { status?: unknown }).status === 'number';
}

/**
 * Players API module
 */
export const playersApi = {
	/**
	 * List all players
	 */
	async list(): Promise<Player[]> {
		return apiClient.get('/players', z.array(PlayerSchema));
	},

	/**
	 * Get a single player by ID
	 */
	async get(id: string): Promise<Player> {
		return apiClient.get(`/players/${id}`, PlayerSchema);
	},

	/**
	 * Create a new player
	 */
	async create(player: Omit<Player, 'id'>): Promise<Player> {
		return apiClient.post('/players', player, PlayerSchema);
	},

	/**
	 * Get players by position
	 */
	async getByPosition(position: Position): Promise<Player[]> {
		return apiClient.get(`/players/position/${position}`, z.array(PlayerSchema));
	},

	/**
	 * Get scouting reports for a player
	 */
	async getScoutingReports(playerId: string): Promise<ScoutingReport[]> {
		return apiClient.get(`/players/${playerId}/scouting-reports`, z.array(ScoutingReportSchema));
	},

	/**
	 * Create a scouting report
	 */
	async createScoutingReport(report: Omit<ScoutingReport, 'id' | 'created_at' | 'updated_at'>): Promise<ScoutingReport> {
		return apiClient.post('/scouting-reports', report, ScoutingReportSchema);
	},

	/**
	 * Get combine results for a player
	 */
	async getCombineResults(playerId: string): Promise<CombineResults | null> {
		try {
			return await apiClient.get(`/players/${playerId}/combine-results`, CombineResultsSchema);
		} catch (error) {
			// Return null if no combine results found (404)
			if (hasStatus(error) && error.status === 404) {
				return null;
			}
			throw error;
		}
	},
};
