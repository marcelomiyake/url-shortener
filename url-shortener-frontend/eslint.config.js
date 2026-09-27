import { defineConfig } from 'eslint/config';
import vue from 'eslint-plugin-vue';
import tseslint from 'typescript-eslint';
import vueParser from 'vue-eslint-parser';

export default defineConfig(
  { ignores: ['coverage/**', 'dist/**', 'node_modules/**', 'playwright-report/**', 'test-results/**'] },
  ...vue.configs['flat/recommended'],
  ...tseslint.configs.recommended,
  {
    files: ['**/*.vue'],
    languageOptions: {
      parser: vueParser,
      parserOptions: {
        parser: tseslint.parser,
        extraFileExtensions: ['.vue'],
      },
    },
  },
);
