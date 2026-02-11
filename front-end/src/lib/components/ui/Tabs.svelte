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

	function handleKeydown(event: KeyboardEvent, currentIndex: number) {
		let newIndex = -1;

		if (event.key === 'ArrowRight') {
			newIndex = (currentIndex + 1) % tabs.length;
		} else if (event.key === 'ArrowLeft') {
			newIndex = (currentIndex - 1 + tabs.length) % tabs.length;
		} else if (event.key === 'Home') {
			newIndex = 0;
		} else if (event.key === 'End') {
			newIndex = tabs.length - 1;
		} else {
			return;
		}

		event.preventDefault();

		const newTab = tabs[newIndex];
		if (!newTab) return;

		onTabChange(newTab.id);

		const button = document.getElementById(`tab-${newTab.id}`) as HTMLButtonElement | null;
		button?.focus();
	}
</script>

<div role="tablist" class="flex border-b border-gray-200 mb-4">
	{#each tabs as tab, index (tab.id)}
		<button
			type="button"
			role="tab"
			id="tab-{tab.id}"
			aria-selected={activeTab === tab.id}
			aria-controls="tabpanel-{tab.id}"
			tabindex={activeTab === tab.id ? 0 : -1}
			class="px-4 py-2 text-sm font-medium border-b-2 transition-colors {activeTab === tab.id
				? 'border-blue-600 text-blue-600'
				: 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'}"
			onclick={() => onTabChange(tab.id)}
			onkeydown={(event) => handleKeydown(event, index)}
		>
			{tab.label}
		</button>
	{/each}
</div>
