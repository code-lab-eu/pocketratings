<script lang="ts">
	const DEFAULT_PLACEHOLDER = 'Search categories and products…';

	let {
		actionUrl = '',
		query = '',
		placeholder = DEFAULT_PLACEHOLDER
	}: {
		actionUrl?: string;
		query?: string;
		placeholder?: string;
	} = $props();

	// Only use actionUrl if it is a safe relative path (prevents open redirect if caller passes a full URL).
	let safeAction = $derived(
		typeof actionUrl === 'string' &&
			actionUrl.startsWith('/') &&
			!actionUrl.startsWith('//')
			? actionUrl
			: '#'
	);
</script>

<form
	action={safeAction}
	method="get"
	class="mb-6"
	role="search"
>
	<label for="search-q" class="sr-only">{placeholder}</label>
	<input
		id="search-q"
		type="search"
		name="q"
		value={query}
		placeholder={placeholder}
		class="pr-input"
		autocomplete="off"
	/>
</form>
