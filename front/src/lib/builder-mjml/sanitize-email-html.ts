import createDOMPurify from 'dompurify'

type DomWindow = Parameters<typeof createDOMPurify>[0]

const ALLOWED_TAGS = [
  'p',
  'br',
  'hr',
  'strong',
  'b',
  'em',
  'i',
  'u',
  's',
  'a',
  'ul',
  'ol',
  'li',
  'h1',
  'h2',
  'h3',
  'h4',
  'h5',
  'h6',
  'blockquote',
  'code',
  'pre',
  'span',
]

const ALLOWED_ATTRIBUTES = ['href', 'target', 'rel', 'style']

const ALLOWED_STYLE_PROPERTIES = new Set([
  'color',
  'font-size',
  'font-weight',
  'font-style',
  'font-family',
  'text-align',
  'line-height',
  'text-decoration',
])

const FORBIDDEN_STYLE_VALUE = /url\s*\(|expression\s*\(|@import|javascript:|behavior|binding|[<>{}\\]/i

function cleanStyle(style: string): string {
  return style
    .split(';')
    .map((declaration) => declaration.trim())
    .flatMap((declaration) => {
      const separator = declaration.indexOf(':')
      if (separator <= 0) {
        return []
      }

      const property = declaration.slice(0, separator).trim().toLowerCase()
      const value = declaration.slice(separator + 1).trim()
      if (!ALLOWED_STYLE_PROPERTIES.has(property) || FORBIDDEN_STYLE_VALUE.test(value)) {
        return []
      }

      return [`${property}: ${value}`]
    })
    .join('; ')
}

export function createEmailHtmlSanitizer(window: DomWindow) {
  const purify = createDOMPurify(window)

  purify.addHook('afterSanitizeAttributes', (node) => {
    const style = node.getAttribute('style')
    if (style !== null) {
      const cleaned = cleanStyle(style)
      if (cleaned === '') {
        node.removeAttribute('style')
      } else {
        node.setAttribute('style', cleaned)
      }
    }

    if (node.tagName === 'A' && node.getAttribute('target') !== null) {
      node.setAttribute('rel', 'noopener noreferrer')
    }
  })

  return (html: string): string =>
    purify.sanitize(html, {
      ALLOWED_TAGS,
      ALLOWED_ATTR: ALLOWED_ATTRIBUTES,
      ALLOW_DATA_ATTR: false,
      ALLOW_ARIA_ATTR: false,
    })
}

let browserSanitizer: ((html: string) => string) | undefined

export function sanitizeEmailHtml(html: string): string {
  browserSanitizer ??= createEmailHtmlSanitizer(window)
  return browserSanitizer(html)
}
