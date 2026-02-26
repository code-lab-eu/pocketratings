<script lang="ts">
	import type { Snippet } from 'svelte';

	type HeadingTag = 'h2' | 'h3' | 'h4';

	interface Props {
		tag?: HeadingTag;
		class?: string;
		children?: Snippet;
	}

	let { tag = 'h2', class: className = '', children }: Props = $props();

	const baseClass = 'pr-heading-section';
	const classes = $derived(className ? `${baseClass} ${className}` : baseClass);
</script>

<svelte:element this={tag} class={classes}>
	{@render children?.()}
</svelte:element>

