// App is always behind auth (JWT in localStorage). Disable SSR so all routes run in the browser with token.
export const ssr = false;
