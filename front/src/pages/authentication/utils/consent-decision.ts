export interface ConsentScopeView {
  name: string
  description: string | null
}

export function defaultApprovedOptionalScopes(
  optionalScopes: readonly ConsentScopeView[],
): Set<string> {
  return new Set(optionalScopes.map((scope) => scope.name))
}

export function toggleOptionalScope(
  approved: ReadonlySet<string>,
  name: string,
): Set<string> {
  const next = new Set(approved)
  if (next.has(name)) {
    next.delete(name)
  } else {
    next.add(name)
  }
  return next
}

export function approvedScopesForAllow(
  optionalScopes: readonly ConsentScopeView[],
  approved: ReadonlySet<string>,
): string[] {
  return optionalScopes.filter((scope) => approved.has(scope.name)).map((scope) => scope.name)
}

export function approvedScopesForDeny(): string[] {
  return []
}

export function approvedScopesFromFormData(
  optionalScopes: readonly ConsentScopeView[],
  data: FormData,
): string[] {
  const checked = new Set(data.getAll('approved_scopes').map(String))
  return optionalScopes.filter((scope) => checked.has(scope.name)).map((scope) => scope.name)
}
