import test from 'node:test'
import assert from 'node:assert/strict'

import {
  darkPaletteCssVars,
  defaultTheme,
  hasOwnSurface,
  mergeWithDefaults,
  outlineColor,
  ownSurfaceTheme,
  themeToCssVars,
  withDarkPalette,
} from './theme.ts'
import type { BuilderNode } from '@/lib/builder-core'

function block(
  type: string,
  props: Record<string, unknown>,
  breakpoints: BuilderNode['breakpoints'] = {},
): BuilderNode {
  return { id: 'b', type, props, breakpoints, styles: {}, children: [] }
}

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

test('a block with a hand-picked background has its own surface', () => {
  assert.equal(hasOwnSurface(block('card', { backgroundColor: '#ffffff' })), true)
  assert.equal(hasOwnSurface(block('container', {}, { md: { backgroundColor: '#fff' } })), true)
})

test('a fully transparent background is not an own surface', () => {
  const clear = [
    'transparent',
    ' TRANSPARENT ',
    'initial',
    'unset',
    'rgba(0,0,0,0)',
    'rgba(255, 255, 255, 0)',
    'rgb(1, 2, 3, 0.0)',
    'rgb(0 0 0 / 0)',
    'rgb(0 0 0 / 0%)',
    'hsla(0, 0%, 0%, 0)',
    'hsl(0 0% 0% / 0)',
    'oklch(0.5 0.1 200 / 0)',
    '#0000',
    '#fff0',
    '#00000000',
    '#FFFFFF00',
  ]
  for (const color of clear) {
    assert.equal(hasOwnSurface(block('card', { backgroundColor: color })), false, color)
    assert.equal(
      hasOwnSurface(block('container', {}, { md: { backgroundColor: color } })),
      false,
      color
    )
  }
})

test('a visible background stays an own surface', () => {
  const visible = [
    '#fff',
    '#ffff',
    '#ffffff',
    '#ffffff80',
    'white',
    'rgb(255, 255, 255)',
    'rgba(255,255,255,0.5)',
    'rgb(255 255 255 / 50%)',
    'hsl(0 0% 100%)',
    'var(--brand)',
  ]
  for (const color of visible) {
    assert.equal(hasOwnSurface(block('card', { backgroundColor: color })), true, color)
  }
})

test('a block that follows the widget background has no own surface', () => {
  assert.equal(hasOwnSurface(block('card', {})), false)
  assert.equal(hasOwnSurface(block('card', { backgroundColor: '' })), false)
  assert.equal(hasOwnSurface(block('text', { backgroundColor: '#ffffff' })), false)
})

test('a hand-picked surface keeps the light palette', () => {
  const light = mergeWithDefaults({ colors: { bodyText: '#1e212a', widgetBackground: '#ffffff' } })
  const own = ownSurfaceTheme(light)
  assert.equal(own.vars['--fk-light-color-body-text'], '#1e212a')
  assert.equal(own.vars['--fk-light-color-widget-bg'], '#ffffff')
  assert.ok(own.css.startsWith('[data-fk-own-surface] {'))
  assert.ok(own.css.includes('--fk-color-body-text: var(--fk-light-color-body-text);'))
  assert.equal(own.css.includes('--fk-font'), false)
})
