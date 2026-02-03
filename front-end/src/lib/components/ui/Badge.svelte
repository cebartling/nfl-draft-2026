<script lang="ts">
	import { clsx } from 'clsx';

	interface Props {
		variant?: 'default' | 'primary' | 'success' | 'warning' | 'danger' | 'info';
		size?: 'sm' | 'md' | 'lg';
		children?: import('svelte').Snippet;
	}

	let {
		variant = 'default',
		size = 'md',
		children,
	}: Props = $props();

	const baseClasses = 'inline-flex items-center font-medium rounded-full';

	const variantClasses = {
		default: 'bg-gray-100 text-gray-800',
		primary: 'bg-blue-100 text-blue-800',
		success: 'bg-green-100 text-green-800',
		warning: 'bg-yellow-100 text-yellow-800',
		danger: 'bg-red-100 text-red-800',
		info: 'bg-indigo-100 text-indigo-800',
	};

	const sizeClasses = {
		sm: 'px-2 py-0.5 text-xs',
		md: 'px-2.5 py-0.5 text-sm',
		lg: 'px-3 py-1 text-base',
	};

	const badgeClasses = $derived(
		clsx(
			baseClasses,
			variantClasses[variant],
			sizeClasses[size]
		)
	);
</script>

<span class={badgeClasses}>
	{#if children}
		{@render children()}
	{/if}
</span>
