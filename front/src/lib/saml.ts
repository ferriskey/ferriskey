import type { Schemas } from '@/api/api.client'
import { translate } from '@/lib/i18n'

export interface SamlOption<TValue extends string> {
  value: TValue
  label: string
  description: string
}

const NAME_ID_FORMAT_KEYS: Record<Schemas.NameIdFormat, string> = {
  'urn:oasis:names:tc:SAML:1.1:nameid-format:emailAddress': 'email',
  'urn:oasis:names:tc:SAML:2.0:nameid-format:persistent': 'persistent',
  'urn:oasis:names:tc:SAML:2.0:nameid-format:transient': 'transient',
  'urn:oasis:names:tc:SAML:1.1:nameid-format:unspecified': 'unspecified',
}

const ATTRIBUTE_NAME_FORMAT_KEYS: Record<Schemas.SamlAttributeNameFormat, string> = {
  'urn:oasis:names:tc:SAML:2.0:attrname-format:basic': 'basic',
  'urn:oasis:names:tc:SAML:2.0:attrname-format:uri': 'uri',
  'urn:oasis:names:tc:SAML:2.0:attrname-format:unspecified': 'unspecified',
}

const samlText = (path: string, options?: Record<string, unknown>) =>
  translate(`common:saml.${path}`, options)

function toOptions<TValue extends string>(
  group: string,
  keys: Record<TValue, string>
): SamlOption<TValue>[] {
  return (Object.keys(keys) as TValue[]).map((value) => ({
    value,
    get label() {
      return samlText(`${group}.${keys[value]}.label`)
    },
    get description() {
      return samlText(`${group}.${keys[value]}.description`)
    },
  }))
}

export const NAME_ID_FORMAT_OPTIONS = toOptions<Schemas.NameIdFormat>(
  'name_id_format',
  NAME_ID_FORMAT_KEYS
)

export const ATTRIBUTE_NAME_FORMAT_OPTIONS = toOptions<Schemas.SamlAttributeNameFormat>(
  'attribute_name_format',
  ATTRIBUTE_NAME_FORMAT_KEYS
)

export const DEFAULT_NAME_ID_FORMAT: Schemas.NameIdFormat =
  'urn:oasis:names:tc:SAML:1.1:nameid-format:emailAddress'

export const DEFAULT_ATTRIBUTE_NAME_FORMAT: Schemas.SamlAttributeNameFormat =
  'urn:oasis:names:tc:SAML:2.0:attrname-format:basic'

const CUSTOM_ATTRIBUTE_PREFIX = 'attribute:'

export const CUSTOM_ATTRIBUTE_SOURCE = 'custom'

const BUILT_IN_SOURCE_KEYS = {
  'user:email': 'email',
  'user:username': 'username',
  'user:first_name': 'first_name',
  'user:last_name': 'last_name',
  'user:id': 'id',
} as const

export type BuiltInAttributeSource = keyof typeof BUILT_IN_SOURCE_KEYS

export const BUILT_IN_SOURCE_OPTIONS = (
  Object.keys(BUILT_IN_SOURCE_KEYS) as BuiltInAttributeSource[]
).map((value) => ({
  value,
  get label() {
    return samlText(`attribute_source.${BUILT_IN_SOURCE_KEYS[value]}`)
  },
}))

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
    return samlText('attribute_source.custom', { key: customAttributeKey(source) })
  }

  const key = BUILT_IN_SOURCE_KEYS[source as BuiltInAttributeSource]
  return key ? samlText(`attribute_source.${key}`) : source
}

export function describeNameIdFormat(format: string): string {
  const key = NAME_ID_FORMAT_KEYS[format as Schemas.NameIdFormat]
  return key ? samlText(`name_id_format.${key}.label`) : format
}

export function describeAttributeNameFormat(format: string): string {
  const key = ATTRIBUTE_NAME_FORMAT_KEYS[format as Schemas.SamlAttributeNameFormat]
  return key ? samlText(`attribute_name_format.${key}.label`) : format
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
