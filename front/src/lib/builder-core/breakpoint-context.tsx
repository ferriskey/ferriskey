import { createContext, useContext, useMemo, useState, type ReactNode } from 'react'
import type { Breakpoint } from './types'

interface BreakpointContextValue {
  /** Currently edited breakpoint, or `null` for the base (default) layer. */
  current: Breakpoint | null
  setCurrent: (bp: Breakpoint | null) => void
}

const BreakpointContext = createContext<BreakpointContextValue | null>(null)

export function BreakpointProvider({ children }: { children: ReactNode }) {
  const [current, setCurrent] = useState<Breakpoint | null>(null)
  const value = useMemo(() => ({ current, setCurrent }), [current])
  return <BreakpointContext.Provider value={value}>{children}</BreakpointContext.Provider>
}

/**
 * Read or change the breakpoint the config panel is currently editing.
 * Safe to call when no provider is present — returns `current: null`
 * (base layer) so consumers that don't care about breakpoints work as-is.
 */
export function useEditingBreakpoint(): BreakpointContextValue {
  const ctx = useContext(BreakpointContext)
  if (ctx) return ctx
  return { current: null, setCurrent: () => {} }
}
