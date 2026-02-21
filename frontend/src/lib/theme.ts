import { writable } from 'svelte/store';

const STORAGE_KEY = 'pocketratings-theme';

function loadDark(): boolean {
	if (typeof window === 'undefined') return false;
	return localStorage.getItem(STORAGE_KEY) === 'dark';
}

function persistDark(value: boolean): void {
	if (typeof window === 'undefined') return;
	localStorage.setItem(STORAGE_KEY, value ? 'dark' : 'light');
}

export const dark = writable<boolean>(false);

let themeInited = false;

/** Call once on client to load theme from localStorage and apply to document. */
export function initTheme(): void {
	if (typeof window === 'undefined' || themeInited) return;
	themeInited = true;
	const initial = loadDark();
	dark.set(initial);
	document.documentElement.classList.toggle('dark', initial);
	dark.subscribe((isDark) => {
		persistDark(isDark);
		document.documentElement.classList.toggle('dark', isDark);
	});
}

export function toggleDark(): void {
	dark.update((v) => !v);
}
