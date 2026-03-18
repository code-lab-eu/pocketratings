import { get } from 'svelte/store';
import { beforeEach, describe, expect, it } from 'vitest';
import { clearToken, getToken, setToken, token } from './auth';

const TOKEN_KEY = 'pocketratings_token';

describe('auth', () => {
  beforeEach(() => {
    clearToken();
  });

  it('setToken sets localStorage and the token store', () => {
    const t = 'eyJhbGciOiJIUzI1NiJ9.test';
    setToken(t);
    expect(localStorage.getItem(TOKEN_KEY)).toBe(t);
    expect(get(token)).toBe(t);
  });

  it('clearToken removes from localStorage and clears the store', () => {
    setToken('some-token');
    clearToken();
    expect(localStorage.getItem(TOKEN_KEY)).toBeNull();
    expect(get(token)).toBeNull();
  });

  it('getToken returns stored value from store', () => {
    setToken('stored');
    expect(getToken()).toBe('stored');
  });

  it('getToken syncs from localStorage when store is empty', () => {
    localStorage.setItem(TOKEN_KEY, 'from-storage');
    expect(getToken()).toBe('from-storage');
    expect(get(token)).toBe('from-storage');
  });
});
