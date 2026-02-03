<script lang="ts">
	import { clsx } from 'clsx';

	interface Props {
		title?: string;
		padding?: 'none' | 'sm' | 'md' | 'lg';
		hover?: boolean;
		clickable?: boolean;
		onclick?: () => void;
		class?: string;
		children?: import('svelte').Snippet;
	}

	let {
		title,
		padding = 'md',
		hover = false,
		clickable = false,
		onclick,
		class: className,
		children,
	}: Props = $props();

	const baseClasses = 'bg-white rounded-lg shadow-md';

	const paddingClasses = {
		none: '',
		sm: 'p-4',
		md: 'p-6',
		lg: 'p-8',
	};

	const hoverClasses = 'hover:shadow-lg transition-shadow cursor-pointer';

	const cardClasses = $derived(
		clsx(
			baseClasses,
			paddingClasses[padding],
			(hover || clickable) && hoverClasses,
			className
		)
	);
</script>

<!-- svelte-ignore a11y_no_noninteractive_tabindex -->
<div
	class={cardClasses}
	role={onclick ? 'button' : undefined}
	tabindex={onclick ? 0 : undefined}
	{onclick}
	onkeydown={(e) => {
		if (onclick && (e.key === 'Enter' || e.key === ' ')) {
			e.preventDefault();
			onclick();
		}
	}}
>
	{#if title}
		<h3 class="text-lg font-semibold text-gray-900 mb-4">{title}</h3>
	{/if}
	{#if children}
		{@render children()}
	{/if}
</div>
