<script lang="ts">
	import { resolve } from '$app/paths';
	import { page } from '$app/stores';

	const productId = $derived($page.url.searchParams.get('product_id') ?? '');
</script>

<main class="mx-auto max-w-2xl px-4 py-8">
	<p class="text-gray-600">Add review (Phase 4). Product: {productId || '(none)'}</p>
	<a href={resolve('/manage')} class="text-gray-600 hover:text-gray-900">← Manage</a>
</main>
