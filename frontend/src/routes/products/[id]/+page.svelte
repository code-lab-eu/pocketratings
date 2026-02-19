<script lang="ts">
	import { get } from 'svelte/store';
	import { page } from '$app/stores';
	import { resolve } from '$app/paths';

	// Phase 3: product detail with reviews and purchase history
	const id = $derived(get(page).params?.id ?? '');
</script>

<main class="mx-auto max-w-2xl px-4 py-8">
	<p class="text-gray-600">Product detail (Phase 3). ID: {id}</p>
	<a href={resolve('/')} class="text-gray-600 hover:text-gray-900">← Home</a>
</main>
