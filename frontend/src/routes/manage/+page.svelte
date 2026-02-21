<script lang="ts">
	import { resolve } from '$app/paths';

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
	<h1 class="mb-4 text-2xl font-semibold text-gray-900">Manage</h1>
	<nav class="mb-6" aria-label="Management areas">
		<ul class="space-y-2">
			{#each links as { path, label } (path)}
				<li>
					<a
						href={resolve(path)}
						class="block rounded-lg border border-gray-200 bg-white px-4 py-3 text-gray-900 hover:bg-gray-50"
					>
						{label}
					</a>
				</li>
			{/each}
		</ul>
	</nav>
	<a href={resolve('/')} class="text-gray-600 hover:text-gray-900">← Home</a>
</main>
