import type { ConfigFieldDef } from '@/pages/client-scope/constants/protocol-mapper-templates'

export const configToStrings = (config: unknown): Record<string, string> => {
  if (!config || typeof config !== 'object') return {}
  return Object.fromEntries(
    Object.entries(config as Record<string, unknown>).map(([key, value]) => [key, String(value)])
  )
}

export const configFromStrings = (values: Record<string, string>): Record<string, unknown> => {
  const out: Record<string, unknown> = {}
  for (const [key, value] of Object.entries(values)) {
    if (value === 'true') out[key] = true
    else if (value === 'false') out[key] = false
    else out[key] = value
  }
  return out
}

export const parseJsonConfig = (raw?: string): Record<string, unknown> => {
  if (!raw || !raw.trim()) return {}
  try {
    return JSON.parse(raw) as Record<string, unknown>
  } catch {
    return {}
  }
}

export const defaultConfigValues = (fields: ConfigFieldDef[]): Record<string, string> =>
  Object.fromEntries(fields.map((field) => [field.key, field.defaultValue ?? '']))

export type MapperEntityKind = 'client' | 'role'

export const MAPPER_ENTITY_FIELDS: Record<string, MapperEntityKind> = {
  'client.id': 'client',
  'included.client.audience': 'client',
  role: 'role',
}
