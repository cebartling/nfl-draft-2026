<script lang="ts">
	import { clsx } from 'clsx';

	interface Props {
		open?: boolean;
		onClose?: () => void;
		width?: 'sm' | 'md' | 'lg' | 'xl' | 'full';
		title?: string;
		children?: import('svelte').Snippet;
	}

	let {
		open = $bindable(false),
		onClose,
		width = 'md',
		title,
		children,
	}: Props = $props();

	let dialogElement: HTMLDivElement;

	const widthClasses = {
		sm: 'max-w-sm',
		md: 'max-w-md',
		lg: 'max-w-lg',
		xl: 'max-w-xl',
		full: 'max-w-full mx-4',
	};

	$effect(() => {
		if (open && dialogElement) {
			dialogElement.focus();
		}
	});

	function handleClose() {
		open = false;
		onClose?.();
	}

	function handleBackdropClick(event: MouseEvent) {
		if (event.target === event.currentTarget) {
			handleClose();
		}
	}

	function handleKeydown(event: KeyboardEvent) {
		if (event.key === 'Escape' && open) {
			handleClose();
		}
	}
</script>

<svelte:window onkeydown={handleKeydown} />

{#if open}
	<div
		bind:this={dialogElement}
		class="fixed inset-0 z-50 flex items-center justify-center overflow-y-auto bg-black bg-opacity-50 animate-fade-in"
		onclick={handleBackdropClick}
		onkeydown={handleKeydown}
		role="dialog"
		aria-modal="true"
		aria-labelledby={title ? 'modal-title' : undefined}
		tabindex="-1"
	>
		<div
			class={clsx(
				'bg-white rounded-lg shadow-xl w-full animate-slide-in',
				widthClasses[width]
			)}
		>
			<div class="flex items-center justify-between p-6 border-b border-gray-200">
				{#if title}
					<h2 id="modal-title" class="text-xl font-semibold text-gray-900">
						{title}
					</h2>
				{:else}
					<div></div>
				{/if}
				<button
					type="button"
					class="text-gray-400 hover:text-gray-600 transition-colors focus:outline-none"
					onclick={handleClose}
					aria-label="Close modal"
				>
					<svg
						class="w-6 h-6"
						fill="none"
						stroke="currentColor"
						viewBox="0 0 24 24"
					>
						<path
							stroke-linecap="round"
							stroke-linejoin="round"
							stroke-width="2"
							d="M6 18L18 6M6 6l12 12"
						/>
					</svg>
				</button>
			</div>
			<div class="p-6">
				{#if children}
					{@render children()}
				{/if}
			</div>
		</div>
	</div>
{/if}
