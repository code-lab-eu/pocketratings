<script lang="ts">
	import { page } from '$app/stores';
	import { resolve } from '$app/paths';

	// Phase 3: implement search results
	const q = $derived($page.url.searchParams.get('q') ?? '');
</script>

<main class="mx-auto max-w-2xl px-4 py-8">
	<p class="text-gray-600">Search (Phase 3). Query: {q || '(none)'}</p>
	<a href={resolve('/')} class="text-gray-600 hover:text-gray-900">← Home</a>
</main>
