import test from 'node:test'
import assert from 'node:assert/strict'

import {
  darkPaletteCssVars,
  defaultTheme,
  mergeWithDefaults,
  outlineColor,
  ownSurfaceCss,
  ownSurfaceTheme,
  themeToCssVars,
  withDarkPalette,
} from './theme.ts'
import type { BuilderNode } from '@/lib/builder-core'

const WIDTHS = { sm: 640, md: 768, lg: 1024, xl: 1280 }
const PALETTE = { apply: 'APPLY', release: 'RELEASE' }
const BASE_RULE = '[data-fk-id="b"] { APPLY }'

function block(
  type: string,
  props: Record<string, unknown>,
  extra: Partial<BuilderNode> = {}
): BuilderNode {
  return { id: 'b', type, props, styles: {}, children: [], ...extra }
}

function surfaceCss(...tree: BuilderNode[]): string {
  return ownSurfaceCss(tree, PALETTE, WIDTHS)
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

test('a block with a hand-picked background gets the light palette at every width', () => {
  assert.equal(surfaceCss(block('card', { backgroundColor: '#ffffff' })), BASE_RULE)
  assert.equal(surfaceCss(block('container', { backgroundColor: '#ffffff' })), BASE_RULE)
})

test('a background that starts at a breakpoint gets the light palette from there', () => {
  const container = block('container', {}, { breakpoints: { md: { backgroundColor: '#ffffff' } } })
  assert.equal(surfaceCss(container), '@media (min-width: 768px) { [data-fk-id="b"] { APPLY } }')
})

test('a background that turns transparent at a breakpoint gives the dark palette back', () => {
  const container = block(
    'container',
    {},
    { breakpoints: { md: { backgroundColor: '#ffffff' }, lg: { backgroundColor: 'transparent' } } }
  )
  assert.equal(
    surfaceCss(container),
    [
      '@media (min-width: 768px) { [data-fk-id="b"] { APPLY } }',
      '@media (min-width: 1024px) { [data-fk-id="b"] { RELEASE } }',
    ].join('\n')
  )
})

test('breakpoint layers apply in width order, whatever their key order', () => {
  const container = block(
    'container',
    {},
    { breakpoints: { lg: { backgroundColor: 'transparent' }, md: { backgroundColor: '#ffffff' } } }
  )
  assert.equal(
    surfaceCss(container),
    [
      '@media (min-width: 768px) { [data-fk-id="b"] { APPLY } }',
      '@media (min-width: 1024px) { [data-fk-id="b"] { RELEASE } }',
    ].join('\n')
  )
})

test('an empty breakpoint value does not clear a background, as in the generated CSS', () => {
  for (const blank of ['', '   ']) {
    const container = block(
      'container',
      { backgroundColor: '#ffffff' },
      { breakpoints: { md: { backgroundColor: blank } } }
    )
    assert.equal(surfaceCss(container), BASE_RULE)
  }
})

test('swapping one visible background for another changes nothing', () => {
  const container = block(
    'container',
    { backgroundColor: '#ffffff' },
    { breakpoints: { md: { backgroundColor: '#eeeeee' } } }
  )
  assert.equal(surfaceCss(container), BASE_RULE)
})

test('a card has no breakpoint background, so none gives it a rule', () => {
  const card = block('card', {}, { breakpoints: { md: { backgroundColor: '#ffffff' } } })
  assert.equal(surfaceCss(card), '')
})

test('the rules of a block come before those of its children', () => {
  const child = block('card', { backgroundColor: '#ffffff' }, { id: 'child' })
  const parent = block('div', { backgroundColor: '#ffffff' }, { children: [child] })
  assert.equal(surfaceCss(parent), `${BASE_RULE}\n[data-fk-id="child"] { APPLY }`)
})

test('an id that is not a plain token gets no rule', () => {
  const unsafe = block('card', { backgroundColor: '#ffffff' }, { id: 'x"] { color: red } [a="' })
  assert.equal(surfaceCss(unsafe), '')
})

test('without a palette to apply, no rule is made', () => {
  const card = block('card', { backgroundColor: '#ffffff' })
  assert.equal(ownSurfaceCss([card], { apply: '', release: '' }, WIDTHS), '')
})

test('a fully transparent background gets no rule', () => {
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
    assert.equal(surfaceCss(block('card', { backgroundColor: color })), '', color)
    assert.equal(
      surfaceCss(block('container', {}, { breakpoints: { md: { backgroundColor: color } } })),
      '',
      color
    )
  }
})

test('a visible background gets a rule', () => {
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
    assert.equal(surfaceCss(block('card', { backgroundColor: color })), BASE_RULE, color)
  }
})

test('a block that follows the widget background gets no rule', () => {
  assert.equal(surfaceCss(block('card', {})), '')
  assert.equal(surfaceCss(block('card', { backgroundColor: '' })), '')
  assert.equal(surfaceCss(block('text', { backgroundColor: '#ffffff' })), '')
})

test('the light palette is kept under light names and mapped back by two declarations', () => {
  const light = mergeWithDefaults({ colors: { bodyText: '#1e212a', widgetBackground: '#ffffff' } })
  const own = ownSurfaceTheme(light)
  assert.equal(own.vars['--fk-light-color-body-text'], '#1e212a')
  assert.equal(own.vars['--fk-light-color-widget-bg'], '#ffffff')
  assert.ok(own.apply.includes('--fk-color-body-text: var(--fk-light-color-body-text);'))
  assert.ok(own.release.includes('--fk-color-body-text: inherit;'))
  assert.equal(`${own.apply}${own.release}`.includes('--fk-font'), false)
})
