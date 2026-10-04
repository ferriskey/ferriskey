import type { FilterField } from '@/components/kit'
import { Schemas } from '@/api/api.client'
import { userRelationSource } from '@/api/user.relation'
import { translate } from '@/lib/i18n'
import { catalogedEventTypes, eventFamilies } from './event-catalogue'

import SecurityEventType = Schemas.SecurityEventType

const EVENT_FAMILIES = ['authentication', 'credentials', 'administration'] as const

const EVENT_STATUSES = ['success', 'failure'] as const

const TARGET_TYPES = ['user', 'client', 'session', 'webhook', 'identity_provider_link'] as const

const FILTERABLE_EVENT_TYPES = (Object.keys(catalogedEventTypes) as SecurityEventType[]).filter(
  (type) => type !== 'unknown'
)

export const securityEventFilterFields = (): FilterField[] => [
  {
    kind: 'text',
    key: 'ip_address',
    label: translate('seawatch:stream.filter_fields.ip_address'),
  },
  {
    kind: 'enum',
    key: 'event_types',
    label: translate('seawatch:stream.filter_fields.event_types'),
    options: [
      ...EVENT_FAMILIES.map((family) => ({
        value: eventFamilies[family].join(','),
        label: translate(`seawatch:stream.filters.${family}`),
      })),
      ...FILTERABLE_EVENT_TYPES.map((type) => ({
        value: type,
        label: translate(`seawatch:event.${type}`),
      })),
    ],
  },
  {
    kind: 'enum',
    key: 'status',
    label: translate('seawatch:stream.filter_fields.status'),
    options: EVENT_STATUSES.map((status) => ({ value: status, label: status })),
  },
  {
    kind: 'enum',
    key: 'target_type',
    label: translate('seawatch:stream.filter_fields.target_type'),
    options: TARGET_TYPES.map((type) => ({ value: type, label: type })),
  },
  {
    kind: 'relation',
    key: 'actor_id',
    label: translate('seawatch:stream.filter_fields.actor_id'),
    relation: userRelationSource,
  },
]
