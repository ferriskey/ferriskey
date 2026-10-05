import { useMemo } from 'react'
import type { PagedListing, PaginationMetadata } from '@/components/kit'
import { Schemas } from '@/api/api.client'
import {
  useGetSecurityEvents,
  useSecurityEventCount,
  type SecurityEventsFilter,
  type SecurityEventsQuery,
} from '@/api/sea-watch.api'
import { useLocalPagedListing } from '@/pages/iam/organization/feature/use-local-paged-listing'
import { JOURNAL_FILTER_KEYS } from './journal-filter-keys'

import SecurityEventType = Schemas.SecurityEventType

export interface EventJournal {
  listing: PagedListing
  events: Schemas.SecurityEvent[]
  pagination: PaginationMetadata | undefined
  isLoading: boolean
  isError: boolean
  firstCount: number
  secondCount: number
}

export function useEventJournal(
  realm: string,
  [first, second]: readonly [SecurityEventType, SecurityEventType],
  filterKeys: readonly string[] = JOURNAL_FILTER_KEYS
): EventJournal {
  const listing = useLocalPagedListing(filterKeys)
  const filter = listing.state.filters as SecurityEventsFilter

  const { data, isLoading, isError } = useGetSecurityEvents({
    realm,
    query: {
      ...listing.apiQuery,
      event_types: filter.event_types || `${first},${second}`,
    } as SecurityEventsQuery,
    keepPrevious: true,
  })

  const firstCount = useSecurityEventCount({ realm, filter: { ...filter, event_types: first } })
  const secondCount = useSecurityEventCount({ realm, filter: { ...filter, event_types: second } })

  const events = useMemo(() => data?.data ?? [], [data])

  return {
    listing,
    events,
    pagination: data?.metadata,
    isLoading,
    isError,
    firstCount: firstCount.count,
    secondCount: secondCount.count,
  }
}
