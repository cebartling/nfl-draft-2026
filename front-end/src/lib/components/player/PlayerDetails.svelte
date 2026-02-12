<script lang="ts">
	import { Badge, LoadingSpinner } from '$components/ui';
	import { playersApi, rankingsApi } from '$api';
	import { logger } from '$lib/utils/logger';
	import type { AvailablePlayer, ScoutingReport, CombineResults, PlayerRanking, RankingSource } from '$types';

	interface Props {
		player: AvailablePlayer;
	}

	let { player }: Props = $props();

	let activeTab = $state<'overview' | 'scouting' | 'combine' | 'rankings'>('overview');
	let scoutingReports = $state<ScoutingReport[]>([]);
	let combineResults = $state<CombineResults | null>(null);
	let playerRankings = $state<PlayerRanking[]>([]);
	let rankingSources = $state<RankingSource[]>([]);
	let isLoadingScouting = $state(false);
	let isLoadingCombine = $state(false);
	let isLoadingRankings = $state(false);
	let rankingsLoaded = $state(false);

	function formatHeight(inches?: number | null): string {
		if (!inches) return 'N/A';
		const feet = Math.floor(inches / 12);
		const remainingInches = inches % 12;
		return `${feet}'${remainingInches}"`;
	}

	function getPositionColor(position: string): 'primary' | 'danger' | 'info' {
		const offensePositions = ['QB', 'RB', 'WR', 'TE', 'OT', 'OG', 'C'];
		const defensePositions = ['DE', 'DT', 'LB', 'CB', 'S'];

		if (offensePositions.includes(position)) return 'primary';
		if (defensePositions.includes(position)) return 'danger';
		return 'info';
	}

	function getFitGradeColor(grade: string): 'success' | 'warning' | 'danger' | 'default' {
		switch (grade) {
			case 'A':
				return 'success';
			case 'B':
				return 'success';
			case 'C':
				return 'warning';
			case 'D':
				return 'danger';
			case 'F':
				return 'danger';
			default:
				return 'default';
		}
	}

	function getSourceAbbreviation(sourceName: string): string {
		const source = rankingSources.find((s) => s.name === sourceName);
		return source?.abbreviation ?? sourceName.slice(0, 2).toUpperCase();
	}

	// Load scouting reports when switching to scouting tab
	$effect(() => {
		if (activeTab === 'scouting' && scoutingReports.length === 0) {
			isLoadingScouting = true;
			playersApi
				.getScoutingReports(player.id)
				.then((reports) => {
					scoutingReports = reports;
				})
				.catch((err) => {
					logger.error('Failed to load scouting reports:', err);
				})
				.finally(() => {
					isLoadingScouting = false;
				});
		}
	});

	// Load combine results when switching to combine tab
	$effect(() => {
		if (activeTab === 'combine' && combineResults === null && !isLoadingCombine) {
			isLoadingCombine = true;
			playersApi
				.getCombineResults(player.id)
				.then((results) => {
					combineResults = results;
				})
				.catch((err) => {
					logger.error('Failed to load combine results:', err);
				})
				.finally(() => {
					isLoadingCombine = false;
				});
		}
	});

	// Load rankings when switching to rankings tab
	$effect(() => {
		if (activeTab === 'rankings' && !rankingsLoaded && !isLoadingRankings) {
			isLoadingRankings = true;
			Promise.all([
				rankingsApi.getPlayerRankings(player.id),
				rankingsApi.listSources(),
			])
				.then(([rankings, sources]) => {
					playerRankings = rankings;
					rankingSources = sources;
					rankingsLoaded = true;
				})
				.catch((err) => {
					logger.error('Failed to load rankings:', err);
				})
				.finally(() => {
					isLoadingRankings = false;
				});
		}
	});
</script>

<div class="bg-white rounded-lg shadow-md overflow-hidden">
	<!-- Header -->
	<div class="bg-gradient-to-r from-blue-600 to-blue-700 px-6 py-8 text-white">
		<div class="flex items-start justify-between">
			<div>
				<h1 class="text-3xl font-bold mb-2">
					{player.first_name}
					{player.last_name}
				</h1>
				<p class="text-blue-100">{player.college || 'N/A'}</p>
			</div>
			<Badge variant={getPositionColor(player.position)} size="lg">
				{player.position}
			</Badge>
		</div>
	</div>

	<!-- Tabs -->
	<div class="border-b border-gray-200">
		<nav class="flex -mb-px">
			<button
				type="button"
				class="px-6 py-4 text-sm font-medium border-b-2 transition-colors {activeTab === 'overview'
					? 'border-blue-600 text-blue-600'
					: 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'}"
				onclick={() => (activeTab = 'overview')}
			>
				Overview
			</button>
			<button
				type="button"
				class="px-6 py-4 text-sm font-medium border-b-2 transition-colors {activeTab === 'rankings'
					? 'border-blue-600 text-blue-600'
					: 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'}"
				onclick={() => (activeTab = 'rankings')}
			>
				Rankings
			</button>
			<button
				type="button"
				class="px-6 py-4 text-sm font-medium border-b-2 transition-colors {activeTab === 'scouting'
					? 'border-blue-600 text-blue-600'
					: 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'}"
				onclick={() => (activeTab = 'scouting')}
			>
				Scouting Reports
			</button>
			<button
				type="button"
				class="px-6 py-4 text-sm font-medium border-b-2 transition-colors {activeTab === 'combine'
					? 'border-blue-600 text-blue-600'
					: 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'}"
				onclick={() => (activeTab = 'combine')}
			>
				Combine Results
			</button>
		</nav>
	</div>

	<!-- Tab Content -->
	<div class="p-6">
		{#if activeTab === 'overview'}
			<div class="space-y-6">
				<div class="grid grid-cols-2 md:grid-cols-3 gap-6">
					<div>
						<p class="text-sm font-medium text-gray-600 mb-1">Height</p>
						<p class="text-lg font-semibold text-gray-900">
							{formatHeight(player.height_inches)}
						</p>
					</div>
					<div>
						<p class="text-sm font-medium text-gray-600 mb-1">Weight</p>
						<p class="text-lg font-semibold text-gray-900">
							{player.weight_pounds ? `${player.weight_pounds} lbs` : 'N/A'}
						</p>
					</div>
					<div>
						<p class="text-sm font-medium text-gray-600 mb-1">Draft Year</p>
						<p class="text-lg font-semibold text-gray-900">{player.draft_year}</p>
					</div>
				</div>

				<div>
					<p class="text-sm font-medium text-gray-600 mb-1">Draft Eligible</p>
					<Badge variant={player.draft_eligible ? 'success' : 'danger'}>
						{player.draft_eligible ? 'Yes' : 'No'}
					</Badge>
				</div>
			</div>
		{:else if activeTab === 'rankings'}
			{#if isLoadingRankings}
				<div class="flex justify-center py-12">
					<LoadingSpinner size="lg" />
				</div>
			{:else if playerRankings.length === 0}
				<p class="text-center text-gray-500 py-12">No rankings available</p>
			{:else}
				<div class="space-y-4">
					<div class="overflow-x-auto">
						<table class="min-w-full divide-y divide-gray-200">
							<thead class="bg-gray-50">
								<tr>
									<th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
										Source
									</th>
									<th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
										Rank
									</th>
									<th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
										Date
									</th>
								</tr>
							</thead>
							<tbody class="bg-white divide-y divide-gray-200">
								{#each playerRankings as ranking (ranking.source_id)}
									<tr>
										<td class="px-6 py-4 whitespace-nowrap">
											<div class="flex items-center gap-2">
												<span class="inline-flex items-center px-1.5 py-0.5 rounded text-xs font-medium bg-purple-100 text-purple-700">
													{getSourceAbbreviation(ranking.source_name)}
												</span>
												<span class="text-sm text-gray-900">{ranking.source_name}</span>
											</div>
										</td>
										<td class="px-6 py-4 whitespace-nowrap">
											<span class="text-lg font-bold text-gray-900">#{ranking.rank}</span>
										</td>
										<td class="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
											{ranking.scraped_at}
										</td>
									</tr>
								{/each}
							</tbody>
						</table>
					</div>
				</div>
			{/if}
		{:else if activeTab === 'scouting'}
			{#if isLoadingScouting}
				<div class="flex justify-center py-12">
					<LoadingSpinner size="lg" />
				</div>
			{:else if scoutingReports.length === 0}
				<p class="text-center text-gray-500 py-12">No scouting reports available</p>
			{:else}
				<div class="space-y-4">
					{#each scoutingReports as report (report.id)}
						<div class="border border-gray-200 rounded-lg p-4">
							<div class="flex items-center justify-between mb-3">
								<Badge variant="info" size="lg">
									Grade: {report.grade}
								</Badge>
								<div class="flex items-center gap-2">
									{#if report.fit_grade}
										<Badge variant={getFitGradeColor(report.fit_grade)} size="sm">
											Fit: {report.fit_grade}
										</Badge>
									{/if}
									{#if report.injury_concern}
										<Badge variant="danger" size="sm">Injury Concern</Badge>
									{/if}
									{#if report.character_concern}
										<Badge variant="warning" size="sm">Character Concern</Badge>
									{/if}
								</div>
							</div>
							{#if report.notes}
								<div>
									<p class="text-sm font-medium text-gray-600 mb-1">Notes</p>
									<p class="text-sm text-gray-900">{report.notes}</p>
								</div>
							{/if}
						</div>
					{/each}
				</div>
			{/if}
		{:else if activeTab === 'combine'}
			{#if isLoadingCombine}
				<div class="flex justify-center py-12">
					<LoadingSpinner size="lg" />
				</div>
			{:else if !combineResults}
				<p class="text-center text-gray-500 py-12">No combine results available</p>
			{:else}
				<div class="grid grid-cols-2 md:grid-cols-3 gap-6">
					{#if combineResults.forty_yard_dash}
						<div>
							<p class="text-sm font-medium text-gray-600 mb-1">40-Yard Dash</p>
							<p class="text-lg font-semibold text-gray-900">
								{combineResults.forty_yard_dash.toFixed(2)}s
							</p>
						</div>
					{/if}
					{#if combineResults.bench_press}
						<div>
							<p class="text-sm font-medium text-gray-600 mb-1">Bench Press</p>
							<p class="text-lg font-semibold text-gray-900">
								{combineResults.bench_press} reps
							</p>
						</div>
					{/if}
					{#if combineResults.vertical_jump}
						<div>
							<p class="text-sm font-medium text-gray-600 mb-1">Vertical Jump</p>
							<p class="text-lg font-semibold text-gray-900">
								{combineResults.vertical_jump}"
							</p>
						</div>
					{/if}
					{#if combineResults.broad_jump}
						<div>
							<p class="text-sm font-medium text-gray-600 mb-1">Broad Jump</p>
							<p class="text-lg font-semibold text-gray-900">
								{combineResults.broad_jump}"
							</p>
						</div>
					{/if}
					{#if combineResults.three_cone_drill}
						<div>
							<p class="text-sm font-medium text-gray-600 mb-1">3-Cone Drill</p>
							<p class="text-lg font-semibold text-gray-900">
								{combineResults.three_cone_drill.toFixed(2)}s
							</p>
						</div>
					{/if}
					{#if combineResults.twenty_yard_shuttle}
						<div>
							<p class="text-sm font-medium text-gray-600 mb-1">20-Yard Shuttle</p>
							<p class="text-lg font-semibold text-gray-900">
								{combineResults.twenty_yard_shuttle.toFixed(2)}s
							</p>
						</div>
					{/if}
				</div>
			{/if}
		{/if}
	</div>
</div>
