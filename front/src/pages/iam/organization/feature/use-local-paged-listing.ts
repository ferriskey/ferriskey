import { startTransition, useCallback, useEffect, useMemo, useRef, useState } from 'react'
import type { PagedListing } from '@/components/kit'
import {
  readListingState,
  toApiQuery,
  writeListingState,
  type ListingState,
  type SortState,
} from '@/components/kit/listing-query-state'

export function useLocalPagedListing(filterKeys: readonly string[], delay = 300): PagedListing {
  const [params, setParams] = useState(() => new URLSearchParams())
  const state = useMemo(() => readListingState(params, filterKeys), [params, filterKeys])
  const [pending, setPending] = useState<Record<string, string>>({})
  const timers = useRef(new Map<string, ReturnType<typeof setTimeout>>())
  const drafts = useMemo(() => ({ ...state.filters, ...pending }), [state.filters, pending])

  const commit = useCallback(
    (patch: Partial<ListingState>) =>
      setParams((current) => writeListingState(current, patch, filterKeys)),
    [filterKeys]
  )

  const settle = useCallback((keys: readonly string[]) => {
    for (const key of keys) {
      const timer = timers.current.get(key)
      if (timer !== undefined) clearTimeout(timer)
      timers.current.delete(key)
    }
    startTransition(() => {
      setPending((current) => {
        const next = { ...current }
        for (const key of keys) delete next[key]
        return next
      })
    })
  }, [])

  useEffect(() => {
    const scheduled = timers.current
    return () => {
      scheduled.forEach(clearTimeout)
      scheduled.clear()
    }
  }, [])

  const setFilter = useCallback(
    (key: string, value: string) => {
      settle([key])
      commit({ filters: { [key]: value } })
    },
    [commit, settle]
  )

  const setFilters = useCallback(
    (patch: Record<string, string>) => {
      settle(Object.keys(patch))
      commit({ filters: patch })
    },
    [commit, settle]
  )

  const setDraft = useCallback(
    (key: string, value: string) => {
      setPending((current) => ({ ...current, [key]: value }))
      const timer = timers.current.get(key)
      if (timer !== undefined) clearTimeout(timer)
      timers.current.set(
        key,
        setTimeout(() => {
          settle([key])
          commit({ filters: { [key]: value } })
        }, delay)
      )
    },
    [commit, settle, delay]
  )

  const setPage = useCallback((page: number) => commit({ page }), [commit])

  const setSort = useCallback((sort: SortState | null) => commit({ sort }), [commit])

  const clearFilters = useCallback(() => {
    settle(filterKeys)
    commit({ filters: Object.fromEntries(filterKeys.map((k) => [k, ''])) })
  }, [commit, settle, filterKeys])

  const apiQuery = useMemo(() => toApiQuery(state), [state])

  return {
    state,
    drafts,
    apiQuery,
    setPage,
    setSort,
    setFilter,
    setFilters,
    setDraft,
    clearFilters,
  }
}
