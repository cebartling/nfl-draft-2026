<script lang="ts">
	import type { ProspectProfile } from '$lib/api';

	interface Props {
		profile: ProspectProfile;
	}

	let { profile }: Props = $props();

	function decodeHeight(raw: string | null): string | null {
		if (!raw || raw.length !== 4) return null;
		const feet = parseInt(raw[0], 10);
		const inches = parseInt(raw.slice(1, 3), 10);
		const eighths = parseInt(raw[3], 10);
		if (Number.isNaN(feet) || Number.isNaN(inches) || Number.isNaN(eighths)) return null;
		return `${feet}'${inches}${eighths > 0 ? ` ${eighths}/8"` : '"'}`;
	}

	let heightDisplay = $derived(decodeHeight(profile.height_raw));
</script>

<section
	class="bg-white rounded-lg shadow border border-gray-200 overflow-hidden"
	data-testid="beast-profile"
>
	<header class="bg-gradient-to-r from-amber-50 to-orange-50 border-b border-amber-200 px-6 py-4">
		<div class="flex items-start justify-between gap-4">
			<div>
				<h2 class="text-xl font-bold text-gray-900">The Beast 2026</h2>
				<p class="text-sm text-gray-600 mt-1">
					Dane Brugler &middot; The Athletic
					<span class="text-gray-400">&middot; scraped {profile.scraped_at}</span>
				</p>
			</div>
			<div class="flex flex-wrap items-center gap-2 justify-end">
				{#if profile.grade_tier}
					<span
						class="inline-flex items-center px-3 py-1 rounded-full text-sm font-semibold bg-amber-100 text-amber-900 border border-amber-300"
					>
						{profile.grade_tier}
					</span>
				{/if}
				{#if profile.overall_rank}
					<span
						class="inline-flex items-center px-3 py-1 rounded-full text-sm font-semibold bg-blue-100 text-blue-900 border border-blue-300"
					>
						OVR #{profile.overall_rank}
					</span>
				{/if}
				<span
					class="inline-flex items-center px-3 py-1 rounded-full text-sm font-semibold bg-gray-100 text-gray-800 border border-gray-300"
				>
					Pos #{profile.position_rank}
				</span>
			</div>
		</div>
	</header>

	<div class="px-6 py-5 space-y-6">
		<!-- Identity row -->
		<dl class="grid grid-cols-2 sm:grid-cols-4 gap-4 text-sm">
			{#if profile.year_class}
				<div>
					<dt class="text-gray-500 uppercase text-xs tracking-wide">Class</dt>
					<dd class="font-semibold text-gray-900">{profile.year_class}</dd>
				</div>
			{/if}
			{#if profile.birthday}
				<div>
					<dt class="text-gray-500 uppercase text-xs tracking-wide">Birthday</dt>
					<dd class="font-semibold text-gray-900">{profile.birthday}</dd>
				</div>
			{/if}
			{#if heightDisplay}
				<div>
					<dt class="text-gray-500 uppercase text-xs tracking-wide">Height</dt>
					<dd class="font-semibold text-gray-900">{heightDisplay}</dd>
				</div>
			{/if}
			{#if profile.jersey_number}
				<div>
					<dt class="text-gray-500 uppercase text-xs tracking-wide">Jersey</dt>
					<dd class="font-semibold text-gray-900">No. {profile.jersey_number}</dd>
				</div>
			{/if}
		</dl>

		{#if profile.nfl_comparison}
			<div
				class="bg-blue-50 border-l-4 border-blue-400 px-4 py-3 rounded-r"
				data-testid="nfl-comparison"
			>
				<p class="text-xs uppercase tracking-wide text-blue-700 font-semibold">NFL Comparison</p>
				<p class="text-lg font-semibold text-blue-900">{profile.nfl_comparison}</p>
			</div>
		{/if}

		{#if profile.summary}
			<div>
				<h3 class="text-sm font-bold text-gray-800 uppercase tracking-wide mb-2">Summary</h3>
				<p class="text-gray-700 leading-relaxed whitespace-pre-line">{profile.summary}</p>
			</div>
		{/if}

		{#if profile.strengths.length || profile.weaknesses.length}
			<div class="grid grid-cols-1 md:grid-cols-2 gap-6">
				{#if profile.strengths.length}
					<div>
						<h3 class="text-sm font-bold text-green-800 uppercase tracking-wide mb-2">
							Strengths
						</h3>
						<ul class="space-y-1 text-sm text-gray-700">
							{#each profile.strengths as item (item)}
								<li class="flex gap-2">
									<span class="text-green-600 font-bold">+</span>
									<span>{item}</span>
								</li>
							{/each}
						</ul>
					</div>
				{/if}
				{#if profile.weaknesses.length}
					<div>
						<h3 class="text-sm font-bold text-red-800 uppercase tracking-wide mb-2">Weaknesses</h3>
						<ul class="space-y-1 text-sm text-gray-700">
							{#each profile.weaknesses as item (item)}
								<li class="flex gap-2">
									<span class="text-red-600 font-bold">−</span>
									<span>{item}</span>
								</li>
							{/each}
						</ul>
					</div>
				{/if}
			</div>
		{/if}

		{#if profile.background}
			<details class="border-t border-gray-200 pt-4">
				<summary
					class="cursor-pointer text-sm font-bold text-gray-800 uppercase tracking-wide hover:text-gray-600"
				>
					Background
				</summary>
				<p class="text-gray-700 leading-relaxed mt-3 whitespace-pre-line">{profile.background}</p>
			</details>
		{/if}
	</div>
</section>
