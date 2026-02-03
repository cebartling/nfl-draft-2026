<script lang="ts">
	import { clsx } from 'clsx';

	interface Props {
		variant?: 'primary' | 'secondary' | 'danger';
		size?: 'sm' | 'md' | 'lg';
		disabled?: boolean;
		loading?: boolean;
		type?: 'button' | 'submit' | 'reset';
		onclick?: (event: MouseEvent) => void;
		children?: import('svelte').Snippet;
	}

	let {
		variant = 'primary',
		size = 'md',
		disabled = false,
		loading = false,
		type = 'button',
		onclick,
		children,
	}: Props = $props();

	const baseClasses = 'font-medium rounded-lg transition-colors focus:outline-none focus:ring-2 focus:ring-offset-2 inline-flex items-center justify-center';

	const variantClasses = {
		primary: 'bg-blue-600 hover:bg-blue-700 text-white focus:ring-blue-500',
		secondary: 'bg-gray-200 hover:bg-gray-300 text-gray-900 focus:ring-gray-500',
		danger: 'bg-red-600 hover:bg-red-700 text-white focus:ring-red-500',
	};

	const sizeClasses = {
		sm: 'py-1.5 px-3 text-sm',
		md: 'py-2 px-4 text-base',
		lg: 'py-3 px-6 text-lg',
	};

	const disabledClasses = 'opacity-50 cursor-not-allowed';

	const buttonClasses = $derived(
		clsx(
			baseClasses,
			variantClasses[variant],
			sizeClasses[size],
			(disabled || loading) && disabledClasses
		)
	);
</script>

<button
	{type}
	class={buttonClasses}
	disabled={disabled || loading}
	onclick={onclick}
>
	{#if loading}
		<svg
			class="animate-spin h-5 w-5 mr-2"
			xmlns="http://www.w3.org/2000/svg"
			fill="none"
			viewBox="0 0 24 24"
		>
			<circle
				class="opacity-25"
				cx="12"
				cy="12"
				r="10"
				stroke="currentColor"
				stroke-width="4"
			></circle>
			<path
				class="opacity-75"
				fill="currentColor"
				d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
			></path>
		</svg>
	{/if}
	{#if children}
		{@render children()}
	{/if}
</button>
