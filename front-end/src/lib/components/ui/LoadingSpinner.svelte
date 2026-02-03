<script lang="ts">
	import { clsx } from 'clsx';

	interface Props {
		size?: 'sm' | 'md' | 'lg';
		color?: 'primary' | 'secondary' | 'white';
	}

	let { size = 'md', color = 'primary' }: Props = $props();

	const sizeClasses = {
		sm: 'h-4 w-4',
		md: 'h-8 w-8',
		lg: 'h-12 w-12',
	};

	const colorClasses = {
		primary: 'text-blue-600',
		secondary: 'text-gray-600',
		white: 'text-white',
	};

	const spinnerClasses = $derived(clsx('animate-spin', sizeClasses[size], colorClasses[color]));
</script>

<div class="flex items-center justify-center" role="status" aria-label="Loading">
	<svg class={spinnerClasses} xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
		<circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"
		></circle>
		<path
			class="opacity-75"
			fill="currentColor"
			d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
		></path>
	</svg>
	<span class="sr-only">Loading...</span>
</div>
