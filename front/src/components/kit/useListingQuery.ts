import { useEffect, useRef, useState } from 'react'
import { useSearchParams } from 'react-router-dom'

export interface ListingQuery {
  search: string
  draft: string
  setDraft: (v: string) => void
  filter: string
  setFilter: (v: string) => void
}

export function useListingQuery(delay = 300): ListingQuery {
  const [params, setParams] = useSearchParams()
  const search = params.get('q') ?? ''
  const filter = params.get('filter') ?? 'all'
  const [draft, setDraft] = useState(search)

  const commit = useRef(setParams)
  useEffect(() => {
    commit.current = setParams
  }, [setParams])

  useEffect(() => {
    if (draft === search) return
    const timer = setTimeout(() => {
      const next = new URLSearchParams(window.location.search)
      if (draft) next.set('q', draft)
      else next.delete('q')
      commit.current(next, { replace: true })
    }, delay)
    return () => clearTimeout(timer)
  }, [draft, search, delay])

  const setFilter = (value: string) => {
    const next = new URLSearchParams(window.location.search)
    if (value && value !== 'all') next.set('filter', value)
    else next.delete('filter')
    setParams(next, { replace: true })
  }

  return { search, draft, setDraft, filter, setFilter }
}
