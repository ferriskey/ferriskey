export interface RealmLabel {
  displayName?: string | null
  name: string
}

export function buildDocumentTitle(pageTitle: string, realm: RealmLabel | null): string {
  if (!realm) return pageTitle

  const label = realm.displayName?.trim() || realm.name

  return `${pageTitle} – ${label}`
}
