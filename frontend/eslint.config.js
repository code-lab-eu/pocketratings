import js from '@eslint/js';
import svelte from 'eslint-plugin-svelte';
import globals from 'globals';
import tseslint from 'typescript-eslint';
import stylistic from '@stylistic/eslint-plugin';
import svelteConfig from './svelte.config.js';

export default tseslint.config(
  js.configs.recommended,
  ...tseslint.configs.recommended,
  ...svelte.configs.recommended,
  {
    languageOptions: {
      globals: {
        ...globals.browser,
        ...globals.node
      }
    }
  },
  {
    plugins: {
      '@stylistic': stylistic
    },
    rules: {
      indent: 'off',
      '@stylistic/indent': ['error', 2],
      '@stylistic/no-tabs': 'error'
    }
  },
  {
    files: ['**/*.svelte'],
    languageOptions: {
      parserOptions: {
        parser: tseslint.parser,
        extraFileExtensions: ['.svelte'],
        svelteConfig
      }
    }
  }
);
