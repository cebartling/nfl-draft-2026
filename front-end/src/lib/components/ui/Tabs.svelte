<script lang="ts">
	interface Tab {
		id: string;
		label: string;
	}

	interface Props {
		tabs: Tab[];
		activeTab: string;
		onTabChange: (id: string) => void;
	}

	let { tabs, activeTab, onTabChange }: Props = $props();
</script>

<div role="tablist" class="flex border-b border-gray-200 mb-4">
	{#each tabs as tab (tab.id)}
		<button
			type="button"
			role="tab"
			id="tab-{tab.id}"
			aria-selected={activeTab === tab.id}
			aria-controls="tabpanel-{tab.id}"
			class="px-4 py-2 text-sm font-medium border-b-2 transition-colors {activeTab === tab.id
				? 'border-blue-600 text-blue-600'
				: 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'}"
			onclick={() => onTabChange(tab.id)}
		>
			{tab.label}
		</button>
	{/each}
</div>
