export type SortOrder = 'asc' | 'desc'

export interface SortState {
  orderBy: string
  order: SortOrder
}

export interface PaginationMetadata {
  page: number
  limit: number
  total: number
  total_pages: number
  first_page: number
  last_page: number
  next_page?: number | null
  prev_page?: number | null
}

export interface ListingState {
  page: number
  limit: number
  sort: SortState | null
  filters: Record<string, string>
}

export const DEFAULT_LIMIT = 20
const MAX_LIMIT = 100

function positiveInt(raw: string | null, fallback: number, max = Number.MAX_SAFE_INTEGER): number {
  if (raw === null || !/^\d+$/.test(raw)) return fallback
  const n = Number(raw)
  return n >= 1 && n <= max ? n : fallback
}

export function readListingState(params: URLSearchParams, filterKeys: readonly string[]): ListingState {
  const orderBy = params.get('order_by')
  const filters: Record<string, string> = {}
  for (const key of filterKeys) {
    const value = params.get(key)
    if (value) filters[key] = value
  }
  return {
    page: positiveInt(params.get('page'), 1),
    limit: positiveInt(params.get('limit'), DEFAULT_LIMIT, MAX_LIMIT),
    sort: orderBy ? { orderBy, order: params.get('order') === 'asc' ? 'asc' : 'desc' } : null,
    filters,
  }
}

export function writeListingState(
  params: URLSearchParams,
  patch: Partial<ListingState>,
  filterKeys: readonly string[],
): URLSearchParams {
  const next = new URLSearchParams(params)
  const resetsPage = patch.filters !== undefined || patch.sort !== undefined

  if (patch.filters !== undefined) {
    for (const key of filterKeys) {
      if (!(key in patch.filters)) continue
      const value = patch.filters[key]
      if (value) next.set(key, value)
      else next.delete(key)
    }
  }

  if (patch.sort !== undefined) {
    if (patch.sort) {
      next.set('order_by', patch.sort.orderBy)
      next.set('order', patch.sort.order)
    } else {
      next.delete('order_by')
      next.delete('order')
    }
  }

  if (patch.limit !== undefined) {
    if (patch.limit === DEFAULT_LIMIT) next.delete('limit')
    else next.set('limit', String(patch.limit))
  }

  const page = resetsPage ? 1 : patch.page
  if (page !== undefined) {
    if (page <= 1) next.delete('page')
    else next.set('page', String(page))
  }

  return next
}

export function nextSort(current: SortState | null, key: string): SortState | null {
  if (!current || current.orderBy !== key) return { orderBy: key, order: 'asc' }
  if (current.order === 'asc') return { orderBy: key, order: 'desc' }
  return null
}

export function toApiQuery(state: ListingState): Record<string, string | number> {
  const query: Record<string, string | number> = { page: state.page, limit: state.limit }
  if (state.sort) {
    query.order_by = state.sort.orderBy
    query.order = state.sort.order
  }
  for (const [key, value] of Object.entries(state.filters)) {
    if (value) query[key] = value
  }
  return query
}
