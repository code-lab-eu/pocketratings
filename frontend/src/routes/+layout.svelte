<script lang="ts">
	import './layout.css';
	import { goto } from '$app/navigation';
	import { page } from '$app/stores';
	import favicon from '$lib/assets/favicon.svg';
	import { clearToken, getToken, token } from '$lib/auth';

	let { children } = $props();

	// Client-only: sync token store from localStorage on load, then redirect if unauthenticated
	$effect(() => {
		if (typeof window === 'undefined') return;
		getToken(); // sync store from localStorage
		const path = $page.url.pathname;
		if (path !== '/login' && !getToken()) {
			goto('/login');
		}
	});

	function handleLogout() {
		clearToken();
		goto('/login');
	}
</script>

<svelte:head><link rel="icon" href={favicon} /></svelte:head>

{#if typeof window !== 'undefined' && $token && $page.url.pathname !== '/login'}
	<header class="border-b border-gray-200 bg-white px-4 py-3">
		<div class="mx-auto flex max-w-2xl items-center justify-between">
			<a href="/" class="text-lg font-semibold text-gray-900">Pocket Ratings</a>
			<button
				type="button"
				onclick={handleLogout}
				class="text-sm text-gray-600 hover:text-gray-900"
			>
				Log out
			</button>
		</div>
	</header>
{/if}

{@render children()}
