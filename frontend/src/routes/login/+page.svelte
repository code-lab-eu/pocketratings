<script lang="ts">
	import { goto } from '$app/navigation';
	import { getToken, setToken } from '$lib/auth';
	import { login } from '$lib/api';

	let email = $state('');
	let password = $state('');
	let error = $state('');
	let loading = $state(false);

	// If already logged in, go home (client-only)
	$effect(() => {
		if (typeof window !== 'undefined' && getToken()) {
			goto('/');
		}
	});

	async function handleSubmit(e: Event) {
		e.preventDefault();
		error = '';
		loading = true;
		try {
			const { token } = await login(email, password);
			setToken(token);
			goto('/');
		} catch (err) {
			error = err instanceof Error ? err.message : 'Login failed';
		} finally {
			loading = false;
		}
	}
</script>

<svelte:head>
	<title>Login — Pocket Ratings</title>
</svelte:head>

<main class="mx-auto max-w-sm px-4 py-12">
	<h1 class="mb-6 text-2xl font-semibold text-gray-900">Pocket Ratings</h1>
	<p class="mb-6 text-gray-600">Sign in to view your categories and product ratings.</p>

	<form onsubmit={handleSubmit} class="space-y-4">
		<div>
			<label for="email" class="mb-1 block text-sm font-medium text-gray-700">Email</label>
			<input
				id="email"
				type="email"
				required
				autocomplete="email"
				bind:value={email}
				class="w-full rounded-md border border-gray-300 px-3 py-2 shadow-sm focus:border-indigo-500 focus:outline-none focus:ring-1 focus:ring-indigo-500"
			/>
		</div>
		<div>
			<label for="password" class="mb-1 block text-sm font-medium text-gray-700">Password</label>
			<input
				id="password"
				type="password"
				required
				autocomplete="current-password"
				bind:value={password}
				class="w-full rounded-md border border-gray-300 px-3 py-2 shadow-sm focus:border-indigo-500 focus:outline-none focus:ring-1 focus:ring-indigo-500"
			/>
		</div>
		{#if error}
			<p class="text-sm text-red-600" role="alert">{error}</p>
		{/if}
		<button
			type="submit"
			disabled={loading}
			class="w-full rounded-md bg-indigo-600 px-4 py-2 text-sm font-medium text-white shadow-sm hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-indigo-500 focus:ring-offset-2 disabled:opacity-50"
		>
			{loading ? 'Signing in…' : 'Sign in'}
		</button>
	</form>
</main>
