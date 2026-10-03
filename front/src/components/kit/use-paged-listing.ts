import { startTransition, useCallback, useEffect, useMemo, useRef, useState } from 'react'
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
  const [pending, setPending] = useState<Record<string, string>>({})
  const timers = useRef(new Map<string, ReturnType<typeof setTimeout>>())
  const drafts = useMemo(() => ({ ...state.filters, ...pending }), [state.filters, pending])

  const commit = useCallback(
    (patch: Partial<ListingState>) => {
      const current = new URLSearchParams(window.location.search)
      setParams(writeListingState(current, patch, filterKeys), { replace: true })
    },
    [setParams, filterKeys],
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
    [commit, settle],
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
        }, delay),
      )
    },
    [commit, settle, delay],
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
    setDraft,
    clearFilters,
  }
}
