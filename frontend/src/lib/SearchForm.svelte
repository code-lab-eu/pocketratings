<script lang="ts">
  const FIELD_LABEL = 'Search';
  const DEFAULT_PLACEHOLDER = 'Search categories and products…';
  const MIN_SEARCH_CHARS = 2;
  const SEARCH_DEBOUNCE_MS = 150;

  let {
    actionUrl = '',
    query = '',
    placeholder = DEFAULT_PLACEHOLDER,
    onQueryChange
  }: {
    actionUrl?: string;
    query?: string;
    placeholder?: string;
    onQueryChange: (q: string) => void;
  } = $props();

  // Local state synced from query; writable $derived does not fit (sync prop + user input).
  // eslint-disable-next-line svelte/prefer-writable-derived -- controlled input with debounced callback
  let inputValue = $state('');
  let debounceHandle: ReturnType<typeof setTimeout> | null = null;

  $effect(() => {
    inputValue = query;
  });

  function scheduleQueryUpdate(value: string) {
    if (debounceHandle != null) {
      clearTimeout(debounceHandle);
      debounceHandle = null;
    }
    debounceHandle = setTimeout(() => {
      debounceHandle = null;
      const trimmed = value.trim();
      if (trimmed.length >= MIN_SEARCH_CHARS) {
        onQueryChange(trimmed);
      } else if (trimmed.length === 0) {
        onQueryChange('');
      }
    }, SEARCH_DEBOUNCE_MS);
  }

  function handleInput(e: Event) {
    const target = e.target as HTMLInputElement;
    const value = target?.value ?? '';
    inputValue = value;
    scheduleQueryUpdate(value);
  }

  $effect(() => () => {
    if (debounceHandle != null) {
      clearTimeout(debounceHandle);
    }
  });

  let safeAction = $derived(
    typeof actionUrl === 'string' &&
      actionUrl.startsWith('/') &&
      !actionUrl.startsWith('//')
      ? actionUrl
      : '#'
  );
</script>

<form
  action={safeAction}
  method="get"
  class="mb-8"
  role="search"
>
  <label for="search-q" class="pr-text-label mb-2 block">{FIELD_LABEL}</label>
  <input
    id="search-q"
    type="search"
    name="q"
    value={inputValue}
    oninput={handleInput}
    placeholder={placeholder}
    class="pr-input pr-input--search"
    autocomplete="off"
  />
</form>
