export interface ResourceOwnerEntry {
  uri: string
  client_id: string
}

export interface OwnerClientCandidate {
  id: string
  client_id: string
  name: string
  public_client: boolean
  registration_source: string
}

export interface OwnerClientOption {
  id: string
  label: string
}

export type OwnersByResource = Record<string, string>

export function ownerClientOptions(clients: readonly OwnerClientCandidate[]): OwnerClientOption[] {
  return clients
    .filter((client) => !client.public_client && client.registration_source === 'admin')
    .map((client) => ({ id: client.id, label: client.name || client.client_id }))
}

export function ownersFromList(entries: readonly ResourceOwnerEntry[]): OwnersByResource {
  return Object.fromEntries(entries.map((entry) => [entry.uri, entry.client_id]))
}

export function ownersToList(
  owners: OwnersByResource,
  resources: readonly string[]
): ResourceOwnerEntry[] {
  return resources.flatMap((uri) => (owners[uri] ? [{ uri, client_id: owners[uri] }] : []))
}

export function ownersChanged(
  pristine: OwnersByResource,
  current: OwnersByResource,
  resources: readonly string[]
): boolean {
  return resources.some((uri) => (pristine[uri] ?? '') !== (current[uri] ?? ''))
}
