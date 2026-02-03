<script lang="ts">
	import '../app.css';
	import Toast from '$components/ui/Toast.svelte';
	import favicon from '$lib/assets/favicon.svg';

	let { children } = $props();

	let mobileMenuOpen = $state(false);
</script>

<svelte:head>
	<link rel="icon" href={favicon} />
	<title>NFL Draft 2026</title>
</svelte:head>

<div class="min-h-screen bg-gray-50">
	<!-- Navigation Bar -->
	<nav class="sticky top-0 z-50 bg-white shadow-md">
		<div class="max-w-7xl mx-auto px-4">
			<div class="flex items-center justify-between h-16">
				<!-- Logo/Title -->
				<div class="flex items-center">
					<a href="/" data-sveltekit-reload class="flex items-center space-x-2">
						<div class="text-2xl font-bold text-blue-600">NFL</div>
						<div class="text-2xl font-bold text-gray-800">Draft 2026</div>
					</a>
				</div>

				<!-- Navigation Links -->
				<div class="hidden md:flex items-center space-x-8">
					<a
						href="/"
						data-sveltekit-reload
						class="text-gray-700 hover:text-blue-600 font-medium transition-colors"
					>
						Home
					</a>
					<a
						href="/drafts"
						data-sveltekit-reload
						class="text-gray-700 hover:text-blue-600 font-medium transition-colors"
					>
						Drafts
					</a>
					<a
						href="/players"
						data-sveltekit-reload
						class="text-gray-700 hover:text-blue-600 font-medium transition-colors"
					>
						Players
					</a>
					<a
						href="/teams"
						data-sveltekit-reload
						class="text-gray-700 hover:text-blue-600 font-medium transition-colors"
					>
						Teams
					</a>
				</div>

				<!-- Mobile Menu Button -->
				<div class="md:hidden">
					<button
						type="button"
						onclick={() => (mobileMenuOpen = !mobileMenuOpen)}
						class="text-gray-700 hover:text-blue-600 focus:outline-none"
						aria-label="Menu"
						aria-expanded={mobileMenuOpen}
					>
						<svg class="h-6 w-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
							<path
								stroke-linecap="round"
								stroke-linejoin="round"
								stroke-width="2"
								d="M4 6h16M4 12h16M4 18h16"
							/>
						</svg>
					</button>
				</div>
			</div>
		</div>

		<!-- Mobile Menu (hidden by default) -->
		{#if mobileMenuOpen}
			<div class="md:hidden border-t border-gray-200">
				<div class="px-2 pt-2 pb-3 space-y-1">
					<a
						href="/"
						data-sveltekit-reload
						onclick={() => (mobileMenuOpen = false)}
						class="block px-3 py-2 rounded-md text-gray-700 hover:bg-blue-50 hover:text-blue-600 font-medium"
					>
						Home
					</a>
					<a
						href="/drafts"
						data-sveltekit-reload
						onclick={() => (mobileMenuOpen = false)}
						class="block px-3 py-2 rounded-md text-gray-700 hover:bg-blue-50 hover:text-blue-600 font-medium"
					>
						Drafts
					</a>
					<a
						href="/players"
						data-sveltekit-reload
						onclick={() => (mobileMenuOpen = false)}
						class="block px-3 py-2 rounded-md text-gray-700 hover:bg-blue-50 hover:text-blue-600 font-medium"
					>
						Players
					</a>
					<a
						href="/teams"
						data-sveltekit-reload
						onclick={() => (mobileMenuOpen = false)}
						class="block px-3 py-2 rounded-md text-gray-700 hover:bg-blue-50 hover:text-blue-600 font-medium"
					>
						Teams
					</a>
				</div>
			</div>
		{/if}
	</nav>

	<!-- Main Content -->
	<main class="max-w-7xl mx-auto px-4 py-8">
		{@render children()}
	</main>

	<!-- Toast Notifications -->
	<Toast />
</div>
