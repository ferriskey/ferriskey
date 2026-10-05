import type { ColumnFilterField } from '@/components/kit'
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

export interface SecurityEventColumnFilters {
  event_type: ColumnFilterField[]
  status: ColumnFilterField[]
  actor: ColumnFilterField[]
  target: ColumnFilterField[]
  ip_address: ColumnFilterField[]
}

export const securityEventColumnFilters = (): SecurityEventColumnFilters => ({
  event_type: [
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
  ],
  status: [
    {
      kind: 'enum',
      key: 'status',
      label: translate('seawatch:stream.filter_fields.status'),
      options: EVENT_STATUSES.map((status) => ({
        value: status,
        label: translate(`seawatch:stream.status_options.${status}`),
      })),
    },
  ],
  actor: [
    {
      kind: 'relation',
      key: 'actor_id',
      label: translate('seawatch:stream.filter_fields.actor_id'),
      relation: userRelationSource,
    },
  ],
  target: [
    {
      kind: 'enum',
      key: 'target_type',
      label: translate('seawatch:stream.filter_fields.target_type'),
      options: TARGET_TYPES.map((type) => ({
        value: type,
        label: translate(`seawatch:stream.target_types.${type}`),
      })),
    },
  ],
  ip_address: [
    {
      kind: 'text',
      key: 'ip_address',
      label: translate('seawatch:stream.filter_fields.ip_address'),
    },
  ],
})
