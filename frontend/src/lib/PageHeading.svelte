<script lang="ts">
	type HeadingTag = 'h1' | 'h2' | 'h3';

	interface Props {
		tag?: HeadingTag;
		class?: string;
	}

	let { tag = 'h1', class: className = '' }: Props = $props();

	const baseClass = 'pr-heading-page';
	const classes = $derived(className ? `${baseClass} ${className}` : baseClass);
</script>

<svelte:element this={tag} class={classes}>
	<slot />
</svelte:element>

{#if $$slots.description}
	<p class="mt-1 pr-text-muted">
		<slot name="description" />
	</p>
{/if}

