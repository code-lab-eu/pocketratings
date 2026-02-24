<script lang="ts">
	import { resolve } from '$app/paths';
	import PageHeading from '$lib/PageHeading.svelte';

	const manageRoutes = [
		'/manage/categories',
		'/manage/locations',
		'/manage/products',
		'/manage/purchases',
		'/manage/reviews'
	] as const;
	const links = [
		{ path: manageRoutes[0], label: 'Categories' },
		{ path: manageRoutes[1], label: 'Locations' },
		{ path: manageRoutes[2], label: 'Products' },
		{ path: manageRoutes[3], label: 'Purchases' },
		{ path: manageRoutes[4], label: 'Reviews' }
	];
</script>

<main class="mx-auto max-w-2xl px-4 py-8">
	<PageHeading>Manage</PageHeading>
	<nav class="mb-6" aria-label="Management areas">
		<ul class="space-y-2">
			{#each links as { path, label } (path)}
				<li>
					<a href={resolve(path)} class="pr-card block">
						{label}
					</a>
				</li>
			{/each}
		</ul>
	</nav>
	<a href={resolve('/')} class="pr-link-muted">← Home</a>
</main>
