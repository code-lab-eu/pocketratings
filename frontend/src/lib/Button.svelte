<script lang="ts">
	type Variant = 'primary' | 'secondary' | 'link';

	interface Props {
		variant?: Variant;
		href?: string;
		type?: 'button' | 'submit' | 'reset';
		disabled?: boolean;
		class?: string;
	}

	let {
		variant = 'primary',
		href,
		type = 'button',
		disabled = false,
		class: className = ''
	}: Props = $props();

	const baseClass = $derived(
		variant === 'secondary'
			? 'pr-btn-secondary'
			: variant === 'link'
				? 'pr-link-inline'
				: 'pr-btn-primary'
	);

	const classes = $derived(className ? `${baseClass} ${className}` : baseClass);
</script>

{#if href}
	<!-- eslint-disable-next-line svelte/no-navigation-without-resolve -- href is already resolved at callsites or may be external; this component cannot safely call resolve() -->
	<a href={href} class={classes}>
		<slot />
	</a>
{:else}
	<button type={type} disabled={disabled} class={classes}>
		<slot />
	</button>
{/if}

