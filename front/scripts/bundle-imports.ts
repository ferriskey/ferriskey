const LEADING_TRIVIA = String.raw`(?:\s|\/\*[\s\S]*?\*\/|\/\/[^\n]*(?:\n|$))*`
const STATIC_IMPORT = new RegExp(
  String.raw`${LEADING_TRIVIA}(?:import|export)\s*(?:[\w$*{},\s]+?\s*from\s*)?(["'])([^"']+)\1\s*;?`,
  'y'
)
const BROWSER_RESOLVABLE = /^(?:\.{1,2}\/|\/|https?:\/\/)/

export function findBareImports(source: string): string[] {
  const specifiers: string[] = []
  STATIC_IMPORT.lastIndex = 0

  for (let match = STATIC_IMPORT.exec(source); match; match = STATIC_IMPORT.exec(source)) {
    specifiers.push(match[2])
  }

  return specifiers.filter((specifier) => !BROWSER_RESOLVABLE.test(specifier))
}
