import { create } from 'zustand'
import { persist } from 'zustand/middleware'

export type UiMode = 'admin' | 'console'

interface State {
  lastVisited: { admin: string | null; console: string | null }
}

interface Actions {
  setLastVisited: (mode: UiMode, path: string) => void
}

const useUiModeStore = create<State & Actions>()(
  persist(
    (set, get) => ({
      lastVisited: { admin: null, console: null },
      setLastVisited: (mode, path) =>
        set({ lastVisited: { ...get().lastVisited, [mode]: path } }),
    }),
    { name: 'ferriskey:ui-mode' },
  ),
)

export default useUiModeStore
