import { useEffect } from 'react'
import { useLocation } from 'react-router'
import useUiModeStore from '@/store/ui-mode.store'
import { deriveModeFromPath } from './use-switch-mode'

/** Updates lastVisited[mode] from the current pathname. Called by each layout. */
export function useTrackLastVisited() {
  const { pathname } = useLocation()
  const setLastVisited = useUiModeStore((s) => s.setLastVisited)

  useEffect(() => {
    setLastVisited(deriveModeFromPath(pathname), pathname)
  }, [pathname, setLastVisited])
}
