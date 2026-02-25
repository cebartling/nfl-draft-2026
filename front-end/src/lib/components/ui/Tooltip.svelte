<script lang="ts">
	interface Props {
		text: string;
		width?: string;
		children?: import('svelte').Snippet;
	}

	let { text, width, children }: Props = $props();

	let visible = $state(false);
	let triggerEl = $state<HTMLSpanElement | null>(null);
	let tooltipStyle = $state('');
	let tooltipId = $derived(`tooltip-${Math.random().toString(36).slice(2, 9)}`);

	function show() {
		visible = true;
		if (triggerEl) {
			const rect = triggerEl.getBoundingClientRect();
			// Position above the trigger, right-aligned to its right edge
			tooltipStyle = `bottom: ${window.innerHeight - rect.top + 8}px; right: ${window.innerWidth - rect.right}px;`;
		}
	}

	function hide() {
		visible = false;
	}
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<span
	class="inline-flex"
	bind:this={triggerEl}
	onmouseenter={show}
	onmouseleave={hide}
	onfocusin={show}
	onfocusout={hide}
	aria-describedby={visible ? tooltipId : undefined}
>
	{#if children}
		{@render children()}
	{/if}
</span>
{#if visible}
	<span
		id={tooltipId}
		role="tooltip"
		class="fixed px-3 py-2 text-xs text-white bg-gray-900 rounded-lg shadow-lg whitespace-normal z-50 pointer-events-none {width ?? 'max-w-xs'}"
		style={tooltipStyle}
	>
		{text}
	</span>
{/if}
