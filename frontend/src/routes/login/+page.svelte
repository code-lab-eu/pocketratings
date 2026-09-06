<script lang="ts">
  import { goto } from '$app/navigation';
  import { page } from '$app/stores';
  import { resolve } from '$app/paths';
  import { getToken, setToken } from '$lib/auth';
  import { login } from '$lib/api';
  import Button from '$lib/Button.svelte';
  import FormError from '$lib/FormError.svelte';
  import InputField from '$lib/InputField.svelte';

  let email = $state('');
  let password = $state('');
  let error = $state('');
  let loading = $state(false);

  /** Inline notice above the form: session timed out, or a password was just reset. */
  const notice = $derived.by(() => {
    const params = $page?.url?.searchParams;
    if (params?.get('expired') === '1') return 'Session expired. Please sign in again.';
    if (params?.get('reset') === '1') return 'Password updated. Please sign in.';
    return null;
  });

  // If already logged in, go home (client-only)
  $effect(() => {
    if (typeof window !== 'undefined' && getToken()) {
      goto(resolve('/'));
    }
  });

  async function handleSubmit(e: Event) {
    e.preventDefault();
    error = '';
    loading = true;
    try {
      const { token } = await login(email, password);
      setToken(token);
      goto(resolve('/'));
    } catch (err) {
      error = err instanceof Error ? err.message : 'Login failed.';
    } finally {
      loading = false;
    }
  }
</script>

<svelte:head>
  <title>Login — Pocket Ratings</title>
</svelte:head>

<main class="mx-auto max-w-sm px-4 py-12">
  <h1 class="pr-heading-page mb-6">Pocket Ratings</h1>
  {#if notice}
    <p class="mb-4 text-sm text-amber-700 dark:text-amber-300" role="alert">{notice}</p>
  {/if}
  <p class="mb-6 pr-text-muted">
    Sign in to view your categories and product ratings.
  </p>

  <form onsubmit={handleSubmit} class="space-y-4">
    <InputField
      id="email"
      label="Email"
      type="email"
      bind:value={email}
      required
      autocomplete="email"
    />
    <InputField
      id="password"
      label="Password"
      type="password"
      bind:value={password}
      required
      autocomplete="current-password"
    />
    <FormError message={error || undefined} />
    <Button type="submit" disabled={loading} class="w-full">
      {loading ? 'Signing in…' : 'Sign in'}
    </Button>
  </form>
</main>
