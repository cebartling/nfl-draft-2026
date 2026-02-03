<script lang="ts">
	import { toastState, ToastType } from '$stores';

	const typeColors = {
		[ToastType.Success]: 'bg-green-500',
		[ToastType.Error]: 'bg-red-500',
		[ToastType.Info]: 'bg-blue-500',
		[ToastType.Warning]: 'bg-yellow-500',
	};

	const typeIcons = {
		[ToastType.Success]: 'M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z',
		[ToastType.Error]: 'M10 14l2-2m0 0l2-2m-2 2l-2-2m2 2l2 2m7-2a9 9 0 11-18 0 9 9 0 0118 0z',
		[ToastType.Info]: 'M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z',
		[ToastType.Warning]: 'M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z',
	};
</script>

<div class="fixed top-4 right-4 z-50 space-y-2 pointer-events-none">
	{#each toastState.toasts as toast (toast.id)}
		<div
			class="pointer-events-auto animate-slide-in {typeColors[toast.type]} text-white px-6 py-4 rounded-lg shadow-lg flex items-center space-x-3 min-w-[300px] max-w-md cursor-pointer"
			role="button"
			onclick={() => toastState.remove(toast.id)}
			onkeydown={(e) => {
				if (e.key === 'Enter' || e.key === ' ') {
					e.preventDefault();
					toastState.remove(toast.id);
				}
			}}
			tabindex="0"
		>
			<svg
				class="w-6 h-6 flex-shrink-0"
				fill="none"
				stroke="currentColor"
				viewBox="0 0 24 24"
			>
				<path
					stroke-linecap="round"
					stroke-linejoin="round"
					stroke-width="2"
					d={typeIcons[toast.type]}
				/>
			</svg>
			<span class="flex-1">{toast.message}</span>
			<button
				type="button"
				class="flex-shrink-0 text-white hover:text-gray-200 transition-colors focus:outline-none"
				onclick={() => toastState.remove(toast.id)}
				aria-label="Close notification"
			>
				<svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
					<path
						stroke-linecap="round"
						stroke-linejoin="round"
						stroke-width="2"
						d="M6 18L18 6M6 6l12 12"
					/>
				</svg>
			</button>
		</div>
	{/each}
</div>
