<script lang="ts">
	type HeadingTag = 'h2' | 'h3' | 'h4';

	interface Props {
		tag?: HeadingTag;
		class?: string;
	}

	let { tag = 'h2', class: className = '' }: Props = $props();

	const baseClass = 'pr-heading-section';
	const classes = $derived(className ? `${baseClass} ${className}` : baseClass);
</script>

<svelte:element this={tag} class={classes}>
	<slot />
</svelte:element>

