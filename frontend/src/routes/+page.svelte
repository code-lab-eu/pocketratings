<script lang="ts">
	import { resolve } from '$app/paths';

	let { data } = $props();
	let categories = $derived(data.categories);
	let error = $derived(data.error);
</script>

<main class="mx-auto max-w-2xl px-4 py-8">
	<form
		action={resolve('/search')}
		method="get"
		class="mb-6"
		role="search"
	>
		<label for="search-q" class="sr-only">Search products</label>
		<input
			id="search-q"
			type="search"
			name="q"
			placeholder="Search products…"
			class="w-full rounded-lg border border-gray-300 px-4 py-2 text-gray-900 placeholder-gray-500 focus:border-gray-500 focus:outline-none focus:ring-1 focus:ring-gray-500"
			autocomplete="off"
		/>
	</form>

	{#if error}
		<p class="text-red-600">{error}</p>
	{:else if categories.length === 0}
		<p class="text-gray-600">No categories yet. Add some from the menu.</p>
	{:else}
		<ul class="space-y-2">
			{#each categories as category (category.id)}
				<li>
					<a
						href={resolve(`/categories/${category.id}`)}
						class="block rounded-lg border border-gray-200 bg-white px-4 py-3 text-gray-900 hover:bg-gray-50"
					>
						{category.name}
					</a>
				</li>
			{/each}
		</ul>
	{/if}
</main>
