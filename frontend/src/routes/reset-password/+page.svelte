<script lang="ts">
  import { goto, invalidateAll } from '$app/navigation';
  import { resolve } from '$app/paths';
  import { resetPassword } from '$lib/api';
  import { clearToken } from '$lib/auth';
  import { errorMessage } from '$lib/utils/formatters';
  import Button from '$lib/Button.svelte';
  import FormError from '$lib/FormError.svelte';
  import PageHeading from '$lib/PageHeading.svelte';
  import PasswordField from '$lib/PasswordField.svelte';

  let { data } = $props();

  let password = $state('');
  let confirmPassword = $state('');
  let error = $state('');
  let submitting = $state(false);

  /** The link is the credential, so this page works whether or not a session is stored. */
  async function handleSubmit(e: Event) {
    e.preventDefault();
    if (!password) {
      error = 'New password is required.';
      return;
    }
    if (!confirmPassword) {
      error = 'Confirm password is required.';
      return;
    }
    if (password !== confirmPassword) {
      error = 'Passwords do not match.';
      return;
    }
    error = '';
    submitting = true;
    try {
      await resetPassword(data.token, password);
      // The password just changed, so any session in this browser (possibly someone else's) is
      // stale: drop it and make the user sign in with the new password.
      clearToken();
      // eslint-disable-next-line svelte/no-navigation-without-resolve -- resolve() + query string; rule only accepts direct resolve()
      goto(`${resolve('/login')}?reset=1`);
    } catch (e) {
      error = errorMessage(e);
    } finally {
      submitting = false;
    }
  }
</script>

<svelte:head>
  <title>Reset password — Pocket Ratings</title>
</svelte:head>

<main class="mx-auto max-w-sm px-4 py-12">
  <PageHeading class="mb-6">Reset password</PageHeading>

  {#if data.status === 'invalid'}
    <p class="pr-text-body">
      This reset link is invalid or has expired. Ask the administrator who sent it for a new link.
    </p>
  {:else if data.status === 'unavailable'}
    <p class="mb-6 pr-text-body">
      The server could not be reached. Your link is still valid, so check your connection and try
      again.
    </p>
    <button type="button" class="pr-btn-primary w-full" onclick={() => invalidateAll()}>
      Try again
    </button>
  {:else}
    <p class="mb-6 pr-text-muted">Choose a new password for your account.</p>

    <!-- novalidate: the page reports the same wording as the API, rather than browser bubbles. -->
    <form onsubmit={handleSubmit} novalidate class="space-y-4">
      <PasswordField
        id="new-password"
        label="New password"
        bind:value={password}
        required
        autocomplete="new-password"
      />
      <PasswordField
        id="confirm-password"
        label="Confirm password"
        bind:value={confirmPassword}
        required
        autocomplete="new-password"
      />
      <FormError message={error || undefined} />
      <Button type="submit" disabled={submitting} class="w-full">
        {submitting ? 'Setting password…' : 'Set password'}
      </Button>
    </form>
  {/if}
</main>
