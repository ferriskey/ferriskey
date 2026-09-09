import { useCallback, useEffect, useState } from 'react'

const STORAGE_KEY = 'ferriskey:next:sidebar-collapsed'

export function useSidebarCollapsed() {
  const [collapsed, setCollapsedState] = useState(
    () => window.localStorage.getItem(STORAGE_KEY) === 'true'
  )

  const setCollapsed = useCallback((v: boolean) => {
    setCollapsedState(v)
    window.localStorage.setItem(STORAGE_KEY, String(v))
  }, [])

  const toggle = useCallback(
    () => setCollapsed(!collapsed),
    [collapsed, setCollapsed]
  )

  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if ((e.metaKey || e.ctrlKey) && e.key === 'b') {
        e.preventDefault()
        toggle()
      }
    }
    window.addEventListener('keydown', onKey)
    return () => window.removeEventListener('keydown', onKey)
  }, [toggle])

  return { collapsed, setCollapsed, toggle }
}
