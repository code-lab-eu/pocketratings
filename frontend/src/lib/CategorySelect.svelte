<script lang="ts">
	import type { Category } from '$lib/types';

	interface Props {
		options: { category: Category; depth: number }[];
		value: string;
		id: string;
		label: string;
		placeholder?: string;
		required?: boolean;
	}

	let { options, value = $bindable(), id, label, placeholder = '', required = false }: Props = $props();
</script>

<div>
	<label
		for={id}
		class="mb-1 block pr-text-label"
	>
		{label}
	</label>
	<select
		{id}
		bind:value
		{required}
		class="pr-input"
		autocomplete="off"
	>
		{#if placeholder}
			<option value="">{placeholder}</option>
		{/if}
		{#each options as { category, depth } (category.id)}
			<option value={category.id}>{'\u00A0'.repeat(depth * 2)}{category.name}</option>
		{/each}
	</select>
</div>
