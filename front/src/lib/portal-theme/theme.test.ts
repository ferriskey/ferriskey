import test from 'node:test'
import assert from 'node:assert/strict'

import {
  darkPaletteCssVars,
  defaultTheme,
  mergeWithDefaults,
  outlineColor,
  themeToCssVars,
  withDarkPalette,
} from './theme.ts'

test('a theme without a dark palette keeps its colours', () => {
  const lightOnly = { colors: { bodyText: '#000000' } }
  assert.equal(withDarkPalette(lightOnly), lightOnly)
  assert.equal(withDarkPalette({ ...lightOnly, darkColors: null }).colors, lightOnly.colors)
})

test('the dark palette replaces the light colours', () => {
  const tokens = {
    colors: { bodyText: '#000000', links: '#123456' },
    darkColors: { pageBackground: '#0a0a0a', bodyText: '#f5fafc' },
  }
  const colors = mergeWithDefaults(withDarkPalette(tokens)).colors
  assert.equal(colors.pageBackground, '#0a0a0a')
  assert.equal(colors.bodyText, '#f5fafc')
  // Missing dark tokens take the defaults, not the light palette.
  assert.equal(colors.links, defaultTheme.colors.links)
  // The light palette itself is left untouched.
  assert.equal(tokens.colors.bodyText, '#000000')
})

test('outlines stay black-based, whatever the body text, until a dark palette is selected', () => {
  const vars = themeToCssVars(mergeWithDefaults({ colors: { bodyText: '#767676' } }))
  assert.equal(vars['--fk-color-outline-ink'], '#000000')
})

test('a selected dark palette draws outlines in its body text colour', () => {
  const vars = darkPaletteCssVars(mergeWithDefaults({ colors: { bodyText: '#f5fafc' } }))
  assert.equal(vars['--fk-color-outline-ink'], '#f5fafc')
})

test('an outline without an ink variable is the black outline the portal always had', () => {
  assert.ok(outlineColor('floatingInput').includes('var(--fk-color-outline-ink, #000000) 18%'))
})
