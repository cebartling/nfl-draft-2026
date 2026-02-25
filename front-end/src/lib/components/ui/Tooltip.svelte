<script lang="ts">
	interface Props {
		text: string;
		children?: import('svelte').Snippet;
	}

	let { text, children }: Props = $props();

	let visible = $state(false);
	let tooltipId = $derived(`tooltip-${Math.random().toString(36).slice(2, 9)}`);
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<span
	class="relative inline-flex"
	onmouseenter={() => (visible = true)}
	onmouseleave={() => (visible = false)}
	onfocusin={() => (visible = true)}
	onfocusout={() => (visible = false)}
	aria-describedby={visible ? tooltipId : undefined}
>
	{#if children}
		{@render children()}
	{/if}
	{#if visible}
		<span
			id={tooltipId}
			role="tooltip"
			class="absolute bottom-full left-1/2 -translate-x-1/2 mb-2 px-3 py-2 text-xs text-white bg-gray-900 rounded-lg shadow-lg max-w-xs whitespace-normal z-50 pointer-events-none"
		>
			{text}
			<span class="absolute top-full left-1/2 -translate-x-1/2 border-4 border-transparent border-t-gray-900"></span>
		</span>
	{/if}
</span>
