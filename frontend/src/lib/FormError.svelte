<script lang="ts">
  interface Props {
    message: string | null | undefined;
  }

  let { message }: Props = $props();
</script>

{#if message}
  <p class="text-red-600 dark:text-red-300" role="alert">{message}</p>
{/if}
