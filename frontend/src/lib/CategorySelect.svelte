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
	<label for={id} class="mb-1 block text-sm font-medium text-gray-700">{label}</label>
	<select
		{id}
		bind:value
		{required}
		class="w-full rounded-lg border border-gray-300 px-3 py-2 text-gray-900"
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
