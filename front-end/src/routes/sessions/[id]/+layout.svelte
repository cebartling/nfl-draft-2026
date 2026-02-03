<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import { page } from '$app/stores';
	import { draftState } from '$stores/draft.svelte';
	import { websocketState } from '$stores/websocket.svelte';

	let { children } = $props();

	// Extract session ID from route params
	let sessionId = $derived($page.params.id!);

	onMount(async () => {
		// Load draft session
		try {
			await draftState.loadSession(sessionId);
			console.log('Draft session loaded:', sessionId);
		} catch (error) {
			console.error('Failed to load draft session:', error);
		}

		// Connect WebSocket for real-time updates
		try {
			websocketState.connect();
			websocketState.subscribeToSession(sessionId);
			console.log('WebSocket connected for session:', sessionId);
		} catch (error) {
			console.error('Failed to connect WebSocket:', error);
		}
	});

	onDestroy(() => {
		// Disconnect WebSocket when leaving session
		websocketState.disconnect();
		console.log('WebSocket disconnected');
	});
</script>

{@render children()}
