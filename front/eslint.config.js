import js from '@eslint/js'
import i18next from 'eslint-plugin-i18next'
import reactHooks from 'eslint-plugin-react-hooks'
import reactRefresh from 'eslint-plugin-react-refresh'
import globals from 'globals'
import tseslint from 'typescript-eslint'

const outOfScopeForTranslation = [
  'src/lib/i18n/**',
  'src/lib/builder-core/**',
  'src/lib/builder-mjml/**',
  'src/lib/builder-portal/**',
  'src/api/portal-theme.api.tsx',
  '**/*.test.{ts,tsx}',
  '**/*.spec.{ts,tsx}',
]

const technicalJsxAttributes = [
  'className',
  'styleName',
  'style',
  'type',
  'key',
  'id',
  'width',
  'height',
  'to',
  'href',
  'src',
  'name',
  'role',
  'variant',
  'size',
  'side',
  'align',
  'tone',
  'target',
  'rel',
  'htmlFor',
  'autoComplete',
  'inputMode',
  'orientation',
  'position',
  'mode',
  'layout',
  'dataKey',
  'path',
  'storageKey',
  'defaultTheme',
  'fill',
  'stroke',
  'strokeDasharray',
  'data-.*',
  'aria-controls',
  'aria-labelledby',
  'aria-describedby',
  'aria-haspopup',
  'aria-hidden',
  'aria-current',
]

const technicalObjectProperties = [
  '[A-Z_-]+',
  'key',
  'id',
  'type',
  'path',
  'to',
  'href',
  'className',
  'variant',
  'tone',
  'color',
  'fill',
  'background',
  'border',
  'padding',
]

export default tseslint.config(
  { ignores: ['dist', 'src/api/api.client.ts', 'src/api/api.tanstack.ts'] },
  {
    extends: [js.configs.recommended, ...tseslint.configs.recommended],
    files: ['**/*.{ts,tsx}'],
    languageOptions: {
      ecmaVersion: 2020,
      globals: globals.browser,
    },
    plugins: {
      'react-hooks': reactHooks,
      'react-refresh': reactRefresh,
    },
    rules: {
      ...reactHooks.configs.recommended.rules,
      semi: ['error', 'never'],
      quotes: ['error', 'single'],
      'jsx-quotes': ['error', 'prefer-single'],
      '@typescript-eslint/no-namespace': 'off',
      'react-refresh/only-export-components': ['warn', { allowConstantExport: true }],
    },
  },
  {
    files: ['src/**/*.{ts,tsx}'],
    ignores: outOfScopeForTranslation,
    plugins: {
      i18next,
    },
    rules: {
      'i18next/no-literal-string': [
        'error',
        {
          framework: 'react',
          mode: 'jsx-only',
          'jsx-attributes': {
            exclude: technicalJsxAttributes,
          },
          'object-properties': {
            exclude: technicalObjectProperties,
          },
        },
      ],
    },
  }
)
