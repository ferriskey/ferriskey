import type { Schemas } from '@/api/api.client'

export interface SamlOption<TValue extends string> {
  value: TValue
  label: string
  description: string
}

const NAME_ID_FORMATS: Record<Schemas.SamlNameIdFormat, Omit<SamlOption<string>, 'value'>> = {
  'urn:oasis:names:tc:SAML:1.1:nameid-format:emailAddress': {
    label: 'Email address',
    description: 'The user is identified by their email address. What most applications expect.',
  },
  'urn:oasis:names:tc:SAML:2.0:nameid-format:persistent': {
    label: 'Persistent',
    description: 'A stable opaque identifier that stays the same across sessions.',
  },
  'urn:oasis:names:tc:SAML:2.0:nameid-format:transient': {
    label: 'Transient',
    description: 'A throwaway identifier, regenerated at every sign-in.',
  },
  'urn:oasis:names:tc:SAML:1.1:nameid-format:unspecified': {
    label: 'Unspecified',
    description: 'Let the application decide how to read the identifier.',
  },
}

const ATTRIBUTE_NAME_FORMATS: Record<
  Schemas.SamlAttributeNameFormat,
  Omit<SamlOption<string>, 'value'>
> = {
  'urn:oasis:names:tc:SAML:2.0:attrname-format:basic': {
    label: 'Basic',
    description: 'Plain attribute names such as email. The usual choice.',
  },
  'urn:oasis:names:tc:SAML:2.0:attrname-format:uri': {
    label: 'URI',
    description: 'Attribute names written as full URIs.',
  },
  'urn:oasis:names:tc:SAML:2.0:attrname-format:unspecified': {
    label: 'Unspecified',
    description: 'Send no format hint to the application.',
  },
}

function toOptions<TValue extends string>(
  formats: Record<TValue, Omit<SamlOption<string>, 'value'>>,
): SamlOption<TValue>[] {
  return (Object.keys(formats) as TValue[]).map((value) => ({ value, ...formats[value] }))
}

export const NAME_ID_FORMAT_OPTIONS = toOptions<Schemas.SamlNameIdFormat>(NAME_ID_FORMATS)

export const ATTRIBUTE_NAME_FORMAT_OPTIONS =
  toOptions<Schemas.SamlAttributeNameFormat>(ATTRIBUTE_NAME_FORMATS)

export const DEFAULT_NAME_ID_FORMAT: Schemas.SamlNameIdFormat =
  'urn:oasis:names:tc:SAML:1.1:nameid-format:emailAddress'

export const DEFAULT_ATTRIBUTE_NAME_FORMAT: Schemas.SamlAttributeNameFormat =
  'urn:oasis:names:tc:SAML:2.0:attrname-format:basic'

const CUSTOM_ATTRIBUTE_PREFIX = 'attribute:'

export const CUSTOM_ATTRIBUTE_SOURCE = 'custom'

const BUILT_IN_SOURCES = {
  'user:email': 'Email',
  'user:username': 'Username',
  'user:first_name': 'First name',
  'user:last_name': 'Last name',
  'user:id': 'User ID',
} as const

export type BuiltInAttributeSource = keyof typeof BUILT_IN_SOURCES

export const BUILT_IN_SOURCE_OPTIONS = (
  Object.keys(BUILT_IN_SOURCES) as BuiltInAttributeSource[]
).map((value) => ({ value, label: BUILT_IN_SOURCES[value] }))

export function isCustomAttributeSource(source: string): boolean {
  return source.startsWith(CUSTOM_ATTRIBUTE_PREFIX)
}

export function customAttributeKey(source: string): string {
  return isCustomAttributeSource(source) ? source.slice(CUSTOM_ATTRIBUTE_PREFIX.length) : ''
}

export function toCustomAttributeSource(key: string): string {
  return `${CUSTOM_ATTRIBUTE_PREFIX}${key.trim()}`
}

export function describeAttributeSource(source: string): string {
  if (isCustomAttributeSource(source)) {
    return `Custom attribute ${customAttributeKey(source)}`
  }
  return BUILT_IN_SOURCES[source as BuiltInAttributeSource] ?? source
}

export function describeNameIdFormat(format: string): string {
  return NAME_ID_FORMATS[format as Schemas.SamlNameIdFormat]?.label ?? format
}

export function describeAttributeNameFormat(format: string): string {
  return ATTRIBUTE_NAME_FORMATS[format as Schemas.SamlAttributeNameFormat]?.label ?? format
}

export interface AttributeMapperDraft {
  name: string
  source: string
}

export const COMMON_PROFILE_MAPPERS: AttributeMapperDraft[] = [
  { name: 'email', source: 'user:email' },
  { name: 'first_name', source: 'user:first_name' },
  { name: 'last_name', source: 'user:last_name' },
]
