import type { Team, Player, Draft, DraftPick, TradeProposal } from '$lib/types';

/**
 * Sample NFL teams for testing
 */
export const mockTeams: Team[] = [
	{
		id: '11111111-1111-1111-1111-111111111111',
		name: 'Patriots',
		abbreviation: 'NE',
		city: 'New England',
		conference: 'AFC',
		division: 'AFC East',
		logo_url: 'https://example.com/ne.png',
	},
	{
		id: '22222222-2222-2222-2222-222222222222',
		name: 'Bills',
		abbreviation: 'BUF',
		city: 'Buffalo',
		conference: 'AFC',
		division: 'AFC East',
	},
	{
		id: '33333333-3333-3333-3333-333333333333',
		name: 'Dolphins',
		abbreviation: 'MIA',
		city: 'Miami',
		conference: 'AFC',
		division: 'AFC East',
	},
	{
		id: '44444444-4444-4444-4444-444444444444',
		name: 'Jets',
		abbreviation: 'NYJ',
		city: 'New York',
		conference: 'AFC',
		division: 'AFC East',
	},
	{
		id: '55555555-5555-5555-5555-555555555555',
		name: 'Ravens',
		abbreviation: 'BAL',
		city: 'Baltimore',
		conference: 'AFC',
		division: 'AFC North',
	},
	{
		id: '66666666-6666-6666-6666-666666666666',
		name: 'Bengals',
		abbreviation: 'CIN',
		city: 'Cincinnati',
		conference: 'AFC',
		division: 'AFC North',
	},
	{
		id: '77777777-7777-7777-7777-777777777777',
		name: 'Browns',
		abbreviation: 'CLE',
		city: 'Cleveland',
		conference: 'AFC',
		division: 'AFC North',
	},
	{
		id: '88888888-8888-8888-8888-888888888888',
		name: 'Steelers',
		abbreviation: 'PIT',
		city: 'Pittsburgh',
		conference: 'AFC',
		division: 'AFC North',
	},
];

/**
 * Sample players for testing
 */
export const mockPlayers: Player[] = [
	{
		id: 'aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa',
		first_name: 'John',
		last_name: 'Quarterback',
		position: 'QB',
		college: 'Alabama',
		height_inches: 76,
		weight_pounds: 220,
		draft_year: 2026,
		draft_eligible: true,
		projected_round: 1,
	},
	{
		id: 'bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb',
		first_name: 'Mike',
		last_name: 'Receiver',
		position: 'WR',
		college: 'Georgia',
		height_inches: 73,
		weight_pounds: 200,
		draft_year: 2026,
		draft_eligible: true,
		projected_round: 1,
	},
	{
		id: 'cccccccc-cccc-cccc-cccc-cccccccccccc',
		first_name: 'Tom',
		last_name: 'Rusher',
		position: 'RB',
		college: 'Ohio State',
		height_inches: 70,
		weight_pounds: 210,
		draft_year: 2026,
		draft_eligible: true,
		projected_round: 2,
	},
	{
		id: 'dddddddd-dddd-dddd-dddd-dddddddddddd',
		first_name: 'Jake',
		last_name: 'Tackler',
		position: 'LB',
		college: 'Michigan',
		height_inches: 74,
		weight_pounds: 240,
		draft_year: 2026,
		draft_eligible: true,
		projected_round: 1,
	},
	{
		id: 'eeeeeeee-eeee-eeee-eeee-eeeeeeeeeeee',
		first_name: 'Chris',
		last_name: 'Cornerback',
		position: 'CB',
		college: 'LSU',
		height_inches: 71,
		weight_pounds: 190,
		draft_year: 2026,
		draft_eligible: true,
		projected_round: 2,
	},
];

/**
 * Sample draft for testing
 */
export const mockDraft: Draft = {
	id: '99999999-9999-9999-9999-999999999999',
	name: '2026 NFL Draft',
	year: 2026,
	status: 'NotStarted',
	rounds: 7,
	picks_per_round: 32,
	total_picks: null,
	created_at: '2026-01-01T00:00:00Z',
	updated_at: '2026-01-01T00:00:00Z',
};

/**
 * Sample draft picks for testing
 */
export const mockDraftPicks: DraftPick[] = [
	{
		id: 'pick-0001',
		draft_id: mockDraft.id,
		round: 1,
		pick_number: 1,
		overall_pick: 1,
		team_id: mockTeams[0].id,
		player_id: undefined,
		picked_at: undefined,
	},
	{
		id: 'pick-0002',
		draft_id: mockDraft.id,
		round: 1,
		pick_number: 2,
		overall_pick: 2,
		team_id: mockTeams[1].id,
		player_id: undefined,
		picked_at: undefined,
	},
	{
		id: 'pick-0003',
		draft_id: mockDraft.id,
		round: 1,
		pick_number: 3,
		overall_pick: 3,
		team_id: mockTeams[2].id,
		player_id: undefined,
		picked_at: undefined,
	},
	{
		id: 'pick-0004',
		draft_id: mockDraft.id,
		round: 1,
		pick_number: 4,
		overall_pick: 4,
		team_id: mockTeams[3].id,
		player_id: undefined,
		picked_at: undefined,
	},
];

/**
 * Sample trade proposal for testing
 */
export const mockTradeProposal: TradeProposal = {
	trade: {
		id: 'trade-1111',
		session_id: 'session-1111',
		from_team_id: mockTeams[0].id,
		to_team_id: mockTeams[1].id,
		status: 'Proposed',
		proposed_at: '2026-01-01T00:00:00Z',
	},
	from_team_picks: [],
	to_team_picks: [],
	from_team_total_value: 0,
	to_team_total_value: 0,
};

/**
 * Helper function to generate multiple draft picks
 */
export function generateDraftPicks(numRounds: number = 7, numTeams: number = 32): DraftPick[] {
	const picks: DraftPick[] = [];
	let overallPick = 1;

	for (let round = 1; round <= numRounds; round++) {
		for (let pickNum = 1; pickNum <= numTeams; pickNum++) {
			const teamIndex = (pickNum - 1) % mockTeams.length;
			picks.push({
				id: `pick-${overallPick.toString().padStart(4, '0')}`,
				draft_id: mockDraft.id,
				round,
				pick_number: pickNum,
				overall_pick: overallPick,
				team_id: mockTeams[teamIndex].id,
				player_id: undefined,
				picked_at: undefined,
			});
			overallPick++;
		}
	}

	return picks;
}

/**
 * Helper function to create a mock draft session
 */
export function createMockSession(
	status: 'NotStarted' | 'InProgress' | 'Paused' | 'Completed' = 'NotStarted'
) {
	return {
		id: 'session-1111',
		draft_id: mockDraft.id,
		status,
		current_pick_number: 1,
		time_per_pick_seconds: 300,
		created_at: '2026-01-01T00:00:00Z',
		updated_at: '2026-01-01T00:00:00Z',
	};
}

/**
 * Helper function to create a complete draft with initialized picks
 */
export function createMockDraftWithPicks(
	year: number = 2026,
	rounds: number = 7,
	picksPerRound: number = 32
): { draft: Draft; picks: DraftPick[] } {
	const draft: Draft = {
		id: `draft-${year}`,
		name: `${year} NFL Draft`,
		year,
		status: 'NotStarted',
		rounds,
		picks_per_round: picksPerRound,
		total_picks: null,
		created_at: '2026-01-01T00:00:00Z',
		updated_at: '2026-01-01T00:00:00Z',
	};

	const picks: DraftPick[] = [];
	let overallPick = 1;

	for (let round = 1; round <= rounds; round++) {
		for (let pickNum = 1; pickNum <= picksPerRound; pickNum++) {
			const teamIndex = (pickNum - 1) % mockTeams.length;
			picks.push({
				id: `pick-${year}-${overallPick.toString().padStart(4, '0')}`,
				draft_id: draft.id,
				round,
				pick_number: pickNum,
				overall_pick: overallPick,
				team_id: mockTeams[teamIndex].id,
				player_id: undefined,
				picked_at: undefined,
			});
			overallPick++;
		}
	}

	return { draft, picks };
}

/**
 * Helper function to simulate picks being made
 */
export function simulatePicks(
	picks: DraftPick[],
	numPicksToMake: number
): DraftPick[] {
	return picks.map((pick, index) => {
		if (index < numPicksToMake) {
			const playerIndex = index % mockPlayers.length;
			return {
				...pick,
				player_id: mockPlayers[playerIndex].id,
				picked_at: new Date(Date.now() - (numPicksToMake - index) * 60000).toISOString(),
			};
		}
		return pick;
	});
}

