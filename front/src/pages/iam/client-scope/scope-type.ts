import type { PillTone } from '@/components/kit'
import { Schemas } from '@/api/api.client'

type ScopeType = Schemas.ScopeType

export const SCOPE_TYPE_TONE: Record<ScopeType, PillTone> = {
  DEFAULT: 'success',
  OPTIONAL: 'info',
  NONE: 'neutral',
}

export const scopeTypeLabelKey = (scopeType: ScopeType) => `scope_type.${scopeType}`

export const SCOPE_TYPE_CHOICES = [
  { value: 'optional', labelKey: 'scope_form.type.options.optional' },
  { value: 'default', labelKey: 'scope_form.type.options.default' },
] as const
