import { create } from 'zustand'
import { persist } from 'zustand/middleware'

export type UiMode = 'admin' | 'product'

interface State {
  mode: UiMode
}

interface Actions {
  setMode: (mode: UiMode) => void
  toggleMode: () => void
}

const useUiModeStore = create<State & Actions>()(
  persist(
    (set, get) => ({
      mode: 'admin',
      setMode: (mode) => set({ mode }),
      toggleMode: () => set({ mode: get().mode === 'admin' ? 'product' : 'admin' }),
    }),
    { name: 'ferriskey:ui-mode' },
  ),
)

export default useUiModeStore
