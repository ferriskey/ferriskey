import { useCallback, useState } from 'react'
import type { PagedListing } from '@/components/kit'
import { usePagedListingWith, type ListingParamsUpdate } from '@/components/kit/use-paged-listing'

export function useLocalPagedListing(filterKeys: readonly string[], delay = 300): PagedListing {
  const [params, setParams] = useState(() => new URLSearchParams())
  const update = useCallback<ListingParamsUpdate>((next) => setParams(next), [])
  return usePagedListingWith(params, update, filterKeys, delay)
}
