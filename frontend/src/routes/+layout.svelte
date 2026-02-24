<script lang="ts">
	import './layout.css';
	import { goto } from '$app/navigation';
	import { resolve } from '$app/paths';
	import { page } from '$app/stores';
	import favicon from '$lib/assets/favicon.svg';
	import { clearToken, getToken, token } from '$lib/auth';
	import { initTheme, dark, toggleDark } from '$lib/theme';

	let { children } = $props();

	// Client-only: init theme from localStorage and sync token
	$effect(() => {
		if (typeof window === 'undefined') return;
		initTheme();
		getToken();
		const path = $page.url.pathname;
		if (path !== '/login' && !getToken()) {
			goto(resolve('/login'));
		}
	});

	function handleLogout() {
		clearToken();
		goto(resolve('/login'));
	}
</script>

<svelte:head><link rel="icon" href={favicon} /></svelte:head>

{#if typeof window !== 'undefined' && $token && $page.url.pathname !== '/login'}
	<header class="border-b border-gray-200 bg-white px-4 py-3 dark:border-gray-700 dark:bg-gray-800">
		<div class="mx-auto flex max-w-2xl min-w-0 items-center justify-between">
			<div class="flex min-h-[44px] min-w-0 items-center gap-3">
				<a
					href={resolve('/manage')}
					class="flex min-h-[44px] min-w-[44px] items-center justify-center text-gray-600 hover:text-gray-900 dark:text-gray-100 dark:hover:text-gray-50"
					aria-label="Menu"
					>☰</a
				>
				<a
					href={resolve('/')}
					class="min-h-[44px] flex items-center break-words text-lg font-semibold text-gray-900 dark:text-gray-50"
					>Pocket Ratings</a
				>
			</div>
			<div class="flex items-center gap-2">
				<button
					type="button"
					onclick={toggleDark}
					class="min-h-[44px] min-w-[44px] px-2 text-sm text-gray-600 hover:text-gray-900 dark:text-gray-100 dark:hover:text-gray-50"
					aria-label={$dark ? 'Switch to light mode' : 'Switch to dark mode'}
					title={$dark ? 'Light mode' : 'Dark mode'}
				>
					{$dark ? '☀' : '☾'}
				</button>
				<button
					type="button"
					onclick={handleLogout}
					class="min-h-[44px] min-w-[44px] px-2 text-sm text-gray-600 hover:text-gray-900 dark:text-gray-100 dark:hover:text-gray-50"
				>
					Log out
				</button>
			</div>
		</div>
	</header>
{/if}

<div class="min-h-screen dark:bg-gray-900 dark:text-gray-50">
	{@render children()}
</div>
