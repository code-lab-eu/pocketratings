<script lang="ts">
	import { resolve } from '$app/paths';
	import { goto } from '$app/navigation';
	import { createLocation } from '$lib/api';
	import PageHeading from '$lib/PageHeading.svelte';
	import Button from '$lib/Button.svelte';

	let name = $state('');
	let submitting = $state(false);
	let error = $state<string | null>(null);

	async function handleSubmit(e: Event) {
		e.preventDefault();
		error = null;
		const n = name.trim();
		if (!n) {
			error = 'Name is required';
			return;
		}
		submitting = true;
		try {
			await createLocation({ name: n });
			await goto(resolve('/manage/locations'), { invalidateAll: true });
		} catch (e) {
			error = e instanceof Error ? e.message : String(e);
		} finally {
			submitting = false;
		}
	}
</script>

<main class="mx-auto max-w-2xl px-4 py-8">
	<nav class="mb-4">
		<a
			href={resolve('/manage/locations')}
			class="pr-link-muted"
			>← Locations</a
		>
	</nav>
	<PageHeading>New location</PageHeading>

	<form onsubmit={handleSubmit} class="space-y-4">
		{#if error}
			<p class="text-red-600 dark:text-red-300">{error}</p>
		{/if}
		<div>
			<label for="name" class="mb-1 block pr-text-label">Name</label>
			<input
				id="name"
				type="text"
				bind:value={name}
				required
				class="pr-input"
				autocomplete="off"
			/>
		</div>
		<div class="flex gap-2">
			<Button type="submit" disabled={submitting} variant="primary">
				{submitting ? 'Creating…' : 'Create'}
			</Button>
			<Button variant="secondary" href={resolve('/manage/locations')}>
				Cancel
			</Button>
		</div>
	</form>
</main>
