import { useCallback, useEffect, useMemo, useRef, useState } from 'react'
import { useSearchParams } from 'react-router-dom'
import {
  readListingState,
  toApiQuery,
  writeListingState,
  type ListingState,
  type SortState,
} from './listing-query-state'

export interface PagedListing {
  state: ListingState
  drafts: Record<string, string>
  apiQuery: Record<string, string | number>
  setPage: (page: number) => void
  setSort: (sort: SortState | null) => void
  setFilter: (key: string, value: string) => void
  setDraft: (key: string, value: string) => void
  clearFilters: () => void
}

export function usePagedListing(filterKeys: readonly string[], delay = 300): PagedListing {
  const [params, setParams] = useSearchParams()
  const state = useMemo(() => readListingState(params, filterKeys), [params, filterKeys])
  const [drafts, setDrafts] = useState<Record<string, string>>(state.filters)
  const filtersKey = JSON.stringify(state.filters)
  const [syncedKey, setSyncedKey] = useState(filtersKey)
  if (syncedKey !== filtersKey) {
    setSyncedKey(filtersKey)
    setDrafts(state.filters)
  }
  const timers = useRef(new Map<string, ReturnType<typeof setTimeout>>())

  const commit = useCallback(
    (patch: Partial<ListingState>) => {
      const current = new URLSearchParams(window.location.search)
      setParams(writeListingState(current, patch, filterKeys), { replace: true })
    },
    [setParams, filterKeys],
  )

  useEffect(() => {
    const pending = timers.current
    return () => pending.forEach(clearTimeout)
  }, [])

  const setFilter = useCallback(
    (key: string, value: string) => {
      setDrafts((d) => ({ ...d, [key]: value }))
      commit({ filters: { [key]: value } })
    },
    [commit],
  )

  const setDraft = useCallback(
    (key: string, value: string) => {
      setDrafts((d) => ({ ...d, [key]: value }))
      const pending = timers.current.get(key)
      if (pending) clearTimeout(pending)
      timers.current.set(
        key,
        setTimeout(() => commit({ filters: { [key]: value } }), delay),
      )
    },
    [commit, delay],
  )

  const apiQuery = useMemo(() => toApiQuery(state), [state])

  return {
    state,
    drafts,
    apiQuery,
    setPage: (page) => commit({ page }),
    setSort: (sort) => commit({ sort }),
    setFilter,
    setDraft,
    clearFilters: () => commit({ filters: Object.fromEntries(filterKeys.map((k) => [k, ''])) }),
  }
}
