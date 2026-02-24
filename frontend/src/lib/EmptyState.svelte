<script lang="ts">
	import { resolve } from '$app/paths';

	interface Props {
		/** Short message when the list or section is empty (e.g. "No categories yet."). */
		message: string;
		/** Optional link text and path. Pass the route path (e.g. '/manage/categories/new'); resolve() is applied. */
		action?: { label: string; href: string };
	}

	let { message, action }: Props = $props();
</script>

<p class="pr-text-muted">{message}</p>
{#if action}
	<p class="mt-2">
		<a
			href={resolve(
				// eslint-disable-next-line @typescript-eslint/no-explicit-any -- resolve() expects typed Route; generated type is not assignable from string so cast is required
				action.href as any
			)}
			class="pr-link-inline"
			>{action.label}</a
		>
	</p>
{/if}
