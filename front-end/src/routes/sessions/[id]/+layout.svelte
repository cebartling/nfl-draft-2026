<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import { page } from '$app/stores';
	import { draftState } from '$stores/draft.svelte';
	import { websocketState } from '$stores/websocket.svelte';
	import { logger } from '$lib/utils/logger';

	let { children } = $props();

	// Extract session ID from route params
	let sessionId = $derived($page.params.id!);

	onMount(async () => {
		// Load draft session
		try {
			await draftState.loadSession(sessionId);
			logger.info('Draft session loaded:', sessionId);
		} catch (error) {
			logger.error('Failed to load draft session:', error);
		}

		// Connect WebSocket for real-time updates
		try {
			websocketState.connect();
			websocketState.subscribeToSession(sessionId);
			logger.info('WebSocket connected for session:', sessionId);
		} catch (error) {
			logger.error('Failed to connect WebSocket:', error);
		}
	});

	onDestroy(() => {
		// Disconnect WebSocket when leaving session
		websocketState.disconnect();
		logger.info('WebSocket disconnected');
	});
</script>

<div class="pb-12">
	{@render children()}
</div>

<footer class="fixed bottom-0 left-0 right-0 z-40 bg-gray-800 border-t border-gray-700">
	<div class="max-w-7xl mx-auto px-4 py-1.5 flex items-center gap-2 text-sm">
		<span class="{websocketState.isConnected ? 'text-green-400' : 'text-red-400'} text-xs">
			{websocketState.isConnected ? '●' : '○'}
		</span>
		<span class="text-gray-300">
			{websocketState.isConnected ? 'Connected' : 'Disconnected'}
		</span>
	</div>
</footer>
