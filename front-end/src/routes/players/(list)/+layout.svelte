<script lang="ts">
	import { page } from '$app/state';

	let { children } = $props();

	const tabs = [
		{ label: 'All Players', href: '/players' },
		{ label: 'Combine Results', href: '/players/combine' },
	];

	let currentPath = $derived(page.url.pathname);
</script>

<div class="space-y-6">
	<!-- Sub-nav tabs -->
	<nav class="border-b border-gray-200">
		<div class="flex space-x-8">
			{#each tabs as tab (tab.href)}
				<a
					href={tab.href}
					class="py-3 px-1 border-b-2 text-sm font-medium transition-colors {currentPath ===
					tab.href
						? 'border-blue-500 text-blue-600'
						: 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'}"
				>
					{tab.label}
				</a>
			{/each}
		</div>
	</nav>

	{@render children()}
</div>
