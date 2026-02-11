import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { teamsApi } from './teams';
import * as client from './client';
import type { Team, TeamNeed, ScoutingReport } from '$lib/types';

describe('teamsApi', () => {
	let mockGet: ReturnType<typeof vi.fn>;
	let mockPost: ReturnType<typeof vi.fn>;
	let mockPut: ReturnType<typeof vi.fn>;

	beforeEach(() => {
		mockGet = vi.fn();
		mockPost = vi.fn();
		mockPut = vi.fn();

		vi.spyOn(client.apiClient, 'get').mockImplementation(mockGet as any);
		vi.spyOn(client.apiClient, 'post').mockImplementation(mockPost as any);
		vi.spyOn(client.apiClient, 'put').mockImplementation(mockPut as any);
	});

	afterEach(() => {
		vi.restoreAllMocks();
	});

	describe('list', () => {
		it('should fetch all teams', async () => {
			const mockTeams: Team[] = [
				{
					id: '1',
					name: 'Patriots',
					abbreviation: 'NE',
					city: 'New England',
					conference: 'AFC',
					division: 'AFC East',
				},
				{
					id: '2',
					name: 'Bills',
					abbreviation: 'BUF',
					city: 'Buffalo',
					conference: 'AFC',
					division: 'AFC East',
				},
			];

			mockGet.mockResolvedValueOnce(mockTeams);

			const result = await teamsApi.list();

			expect(mockGet).toHaveBeenCalledWith('/teams', expect.any(Object));
			expect(result).toEqual(mockTeams);
		});

		it('should handle empty team list', async () => {
			mockGet.mockResolvedValueOnce([]);

			const result = await teamsApi.list();

			expect(result).toEqual([]);
		});
	});

	describe('get', () => {
		it('should fetch a single team by ID', async () => {
			const mockTeam: Team = {
				id: '123',
				name: 'Patriots',
				abbreviation: 'NE',
				city: 'New England',
				conference: 'AFC',
				division: 'AFC East',
			};

			mockGet.mockResolvedValueOnce(mockTeam);

			const result = await teamsApi.get('123');

			expect(mockGet).toHaveBeenCalledWith('/teams/123', expect.any(Object));
			expect(result).toEqual(mockTeam);
		});

		it('should throw error for non-existent team', async () => {
			mockGet.mockRejectedValueOnce(new client.ApiClientError('Not found', 404));

			await expect(teamsApi.get('999')).rejects.toThrow('Not found');
		});
	});

	describe('create', () => {
		it('should create a new team', async () => {
			const newTeam: Omit<Team, 'id'> = {
				name: 'Patriots',
				abbreviation: 'NE',
				city: 'New England',
				conference: 'AFC',
				division: 'AFC East',
			};

			const createdTeam: Team = {
				id: '123',
				...newTeam,
			};

			mockPost.mockResolvedValueOnce(createdTeam);

			const result = await teamsApi.create(newTeam);

			expect(mockPost).toHaveBeenCalledWith('/teams', newTeam, expect.any(Object));
			expect(result).toEqual(createdTeam);
		});

		it('should throw error for invalid team data', async () => {
			const invalidTeam = {
				name: 'Patriots',
				// Missing required fields
			} as Omit<Team, 'id'>;

			mockPost.mockRejectedValueOnce(new client.ApiClientError('Bad Request', 400));

			await expect(teamsApi.create(invalidTeam)).rejects.toThrow('Bad Request');
		});
	});

	describe('update', () => {
		it('should update an existing team', async () => {
			const teamId = '123';
			const updates: Partial<Omit<Team, 'id'>> = {
				name: 'Updated Patriots',
			};

			const updatedTeam: Team = {
				id: teamId,
				name: 'Updated Patriots',
				abbreviation: 'NE',
				city: 'New England',
				conference: 'AFC',
				division: 'AFC East',
			};

			mockPut.mockResolvedValueOnce(updatedTeam);

			const result = await teamsApi.update(teamId, updates);

			expect(mockPut).toHaveBeenCalledWith(`/teams/${teamId}`, updates, expect.any(Object));
			expect(result).toEqual(updatedTeam);
		});

		it('should throw error for non-existent team', async () => {
			mockPut.mockRejectedValueOnce(new client.ApiClientError('Not found', 404));

			await expect(teamsApi.update('999', { name: 'Test' })).rejects.toThrow('Not found');
		});
	});

	describe('getNeeds', () => {
		it('should fetch team needs', async () => {
			const teamId = '123';
			const mockNeeds: TeamNeed[] = [
				{
					id: '1',
					team_id: teamId,
					position: 'QB',
					priority: 1,
				},
				{
					id: '2',
					team_id: teamId,
					position: 'WR',
					priority: 2,
				},
			];

			mockGet.mockResolvedValueOnce(mockNeeds);

			const result = await teamsApi.getNeeds(teamId);

			expect(mockGet).toHaveBeenCalledWith(`/teams/${teamId}/needs`, expect.any(Object));
			expect(result).toEqual(mockNeeds);
		});

		it('should handle team with no needs', async () => {
			mockGet.mockResolvedValueOnce([]);

			const result = await teamsApi.getNeeds('123');

			expect(result).toEqual([]);
		});
	});

	describe('createNeed', () => {
		it('should create a new team need', async () => {
			const newNeed: Omit<TeamNeed, 'id'> = {
				team_id: '123',
				position: 'QB',
				priority: 1,
			};

			const createdNeed: TeamNeed = {
				id: '456',
				...newNeed,
			};

			mockPost.mockResolvedValueOnce(createdNeed);

			const result = await teamsApi.createNeed(newNeed);

			expect(mockPost).toHaveBeenCalledWith('/teams/needs', newNeed, expect.any(Object));
			expect(result).toEqual(createdNeed);
		});

		it('should throw error for invalid need data', async () => {
			const invalidNeed = {
				team_id: '123',
				position: 'QB',
				// Missing priority
			} as Omit<TeamNeed, 'id'>;

			mockPost.mockRejectedValueOnce(new client.ApiClientError('Bad Request', 400));

			await expect(teamsApi.createNeed(invalidNeed)).rejects.toThrow('Bad Request');
		});
	});

	describe('getScoutingReports', () => {
		it('should fetch scouting reports for a team', async () => {
			const teamId = '123';
			const mockReports: ScoutingReport[] = [
				{
					id: 'r1',
					player_id: 'p1',
					team_id: teamId,
					grade: 92,
					fit_grade: 'A',
					injury_concern: false,
					character_concern: false,
				},
				{
					id: 'r2',
					player_id: 'p2',
					team_id: teamId,
					grade: 78,
					fit_grade: 'B',
					injury_concern: true,
					character_concern: false,
				},
			];

			mockGet.mockResolvedValueOnce(mockReports);

			const result = await teamsApi.getScoutingReports(teamId);

			expect(mockGet).toHaveBeenCalledWith(`/teams/${teamId}/scouting-reports`, expect.any(Object));
			expect(result).toEqual(mockReports);
		});

		it('should handle team with no scouting reports', async () => {
			mockGet.mockResolvedValueOnce([]);

			const result = await teamsApi.getScoutingReports('123');

			expect(result).toEqual([]);
		});
	});
});
