<script lang="ts">
	import { Badge } from '$components/ui';
	import type { Team } from '$types';

	interface Props {
		team: Team;
		onSelect?: (team: Team) => void;
	}

	let { team, onSelect }: Props = $props();

	function getConferenceColor(conference: string): 'primary' | 'danger' {
		return conference === 'AFC' ? 'primary' : 'danger';
	}
</script>

<div
	class="bg-white rounded-lg shadow-md p-4 transition-all {onSelect
		? 'hover:shadow-lg cursor-pointer'
		: ''}"
	onclick={() => onSelect?.(team)}
	role={onSelect ? 'button' : 'article'}
	tabindex={onSelect ? 0 : undefined}
	onkeydown={(e) => {
		if (onSelect && (e.key === 'Enter' || e.key === ' ')) {
			e.preventDefault();
			onSelect(team);
		}
	}}
>
	<div class="flex items-start justify-between mb-3">
		<div class="flex-1">
			<h3 class="text-lg font-semibold text-gray-900">
				{team.city} {team.name}
			</h3>
			<p class="text-sm text-gray-600">{team.abbreviation}</p>
		</div>
		{#if team.logo_url}
			<img src={team.logo_url} alt="{team.name} logo" class="w-12 h-12 object-contain" />
		{/if}
	</div>

	<div class="flex items-center space-x-2">
		<Badge variant={getConferenceColor(team.conference)} size="sm">
			{team.conference}
		</Badge>
		<Badge variant="default" size="sm">
			{team.division}
		</Badge>
	</div>
</div>
