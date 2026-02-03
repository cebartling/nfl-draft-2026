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

{@render children()}
