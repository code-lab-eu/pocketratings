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

<p class="text-gray-600 dark:text-gray-400">{message}</p>
{#if action}
	<p class="mt-2">
		<a
			href={resolve(
				// eslint-disable-next-line @typescript-eslint/no-explicit-any -- dynamic route path from caller
				action.href as any
			)}
			class="text-gray-900 underline hover:no-underline dark:text-gray-200 dark:hover:text-white"
			>{action.label}</a
		>
	</p>
{/if}
