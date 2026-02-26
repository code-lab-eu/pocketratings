<script lang="ts">
	import type { Snippet } from 'svelte';

	type HeadingTag = 'h1' | 'h2' | 'h3';

	interface Props {
		tag?: HeadingTag;
		class?: string;
		children?: Snippet;
		description?: Snippet;
	}

	let { tag = 'h1', class: className = '', children, description }: Props = $props();

	const baseClass = 'pr-heading-page';
	const classes = $derived(className ? `${baseClass} ${className}` : baseClass);
</script>

<svelte:element this={tag} class={classes}>
	{@render children?.()}
</svelte:element>

{#if description}
	<p class="mt-1 pr-text-muted">
		{@render description()}
	</p>
{/if}

