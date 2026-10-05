import { keepPreviousData } from '@tanstack/react-query'

export interface PagedQueryOptions {
  keepPrevious?: boolean
}

export function previousPagePlaceholder(keepPrevious = false) {
  return keepPrevious ? keepPreviousData : undefined
}
