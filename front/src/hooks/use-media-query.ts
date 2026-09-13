import { useCallback, useSyncExternalStore } from 'react'

export const BREAKPOINTS = { md: '48rem', lg: '64rem' } as const

export type LayoutTier = 'phone' | 'tablet' | 'desktop'

export function useMediaQuery(query: string) {
  const subscribe = useCallback(
    (onStoreChange: () => void) => {
      const list = window.matchMedia(query)
      list.addEventListener('change', onStoreChange)
      return () => list.removeEventListener('change', onStoreChange)
    },
    [query],
  )

  const getSnapshot = useCallback(() => window.matchMedia(query).matches, [query])

  return useSyncExternalStore(subscribe, getSnapshot, () => false)
}

export function useLayoutTier(): LayoutTier {
  const atLeastMd = useMediaQuery(`(min-width: ${BREAKPOINTS.md})`)
  const atLeastLg = useMediaQuery(`(min-width: ${BREAKPOINTS.lg})`)

  if (atLeastLg) return 'desktop'
  if (atLeastMd) return 'tablet'
  return 'phone'
}
