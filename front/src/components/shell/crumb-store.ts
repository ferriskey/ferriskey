import { useEffect } from 'react'
import { create } from 'zustand'

interface CrumbState {
  labels: Record<string, string>
  setLabel: (segment: string, label: string) => void
  removeLabel: (segment: string) => void
}

export const useCrumbStore = create<CrumbState>((set) => ({
  labels: {},
  setLabel: (segment, label) =>
    set((state) =>
      state.labels[segment] === label
        ? state
        : { labels: { ...state.labels, [segment]: label } }
    ),
  removeLabel: (segment) =>
    set((state) => {
      if (!(segment in state.labels)) return state
      const next = { ...state.labels }
      delete next[segment]
      return { labels: next }
    }),
}))

export function useCrumbLabel(segment?: string, label?: string | null) {
  useEffect(() => {
    if (!segment || !label) return
    const { setLabel, removeLabel } = useCrumbStore.getState()
    setLabel(segment, label)
    return () => removeLabel(segment)
  }, [segment, label])
}
