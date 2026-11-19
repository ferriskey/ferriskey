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

const storedWithDarkPalette = {
  colors: { bodyText: '#000000' },
  darkColors: { pageBackground: '#0a0a0a', bodyText: '#f5fafc' },
}

test('the editor seed keeps the stored dark palette as it is', () => {
  const seed = mergeWithDefaults(storedWithDarkPalette)
  assert.ok('darkColors' in seed)
  assert.deepEqual(seed.darkColors, storedWithDarkPalette.darkColors)
})

test('saving after a light colour edit still sends the dark palette', () => {
  const seed = mergeWithDefaults(storedWithDarkPalette)
  // The editor sets a colour like this.
  // It sends its whole state on save.
  const edited = { ...seed, colors: { ...seed.colors, bodyText: '#111111' } }
  assert.ok('darkColors' in edited)
  assert.deepEqual(edited.darkColors, storedWithDarkPalette.darkColors)
  assert.equal(edited.colors.bodyText, '#111111')
})

test('a theme without a dark palette seeds the editor without the key', () => {
  for (const stored of [{ colors: { bodyText: '#000000' } }, { darkColors: null }, {}]) {
    assert.equal('darkColors' in mergeWithDefaults(stored), false)
  }
})
