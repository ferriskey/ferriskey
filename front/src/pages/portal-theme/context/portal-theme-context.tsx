import { createContext, useCallback, useContext, useMemo, useState, type ReactNode } from 'react'
import { defaultTheme, mergeWithDefaults, type PortalThemeConfig } from '../lib/theme'
import type { Schemas } from '@/api/api.client'

/**
 * Theme builder sidebar is now organised by component rather than by token
 * type — admins think "I want to customise my buttons", not "I want to edit
 * the colors palette then jump to borders then to fonts". Each tab pulls the
 * relevant subset of color / font / border / spacing tokens into one place.
 */
export type BuilderTab =
  | 'buttons'
  | 'inputs'
  | 'widget'
  | 'typography'
  | 'page'

type PortalThemeContextValue = {
  theme: PortalThemeConfig
  savedTheme: PortalThemeConfig
  isDirty: boolean
  activeTab: BuilderTab
  setActiveTab: (tab: BuilderTab) => void
  setColor: <K extends keyof PortalThemeConfig['colors']>(
    key: K,
    value: PortalThemeConfig['colors'][K],
  ) => void
  setFont: <K extends keyof PortalThemeConfig['fonts']>(
    key: K,
    value: PortalThemeConfig['fonts'][K],
  ) => void
  setBorder: <K extends keyof PortalThemeConfig['borders']>(
    key: K,
    value: PortalThemeConfig['borders'][K],
  ) => void
  setSpacing: <K extends keyof PortalThemeConfig['spacing']>(
    key: K,
    value: PortalThemeConfig['spacing'][K],
  ) => void
  discard: () => void
  markSaved: (saved: PortalThemeConfig) => void
}

const PortalThemeContext = createContext<PortalThemeContextValue | null>(null)

export function PortalThemeProvider({
  initial,
  children,
}: {
  initial: Schemas.PortalThemeConfig | undefined
  children: ReactNode
}) {
  const seed = useMemo(() => mergeWithDefaults(initial), [initial])
  const [theme, setTheme] = useState<PortalThemeConfig>(seed)
  const [savedTheme, setSavedTheme] = useState<PortalThemeConfig>(seed)
  const [activeTab, setActiveTab] = useState<BuilderTab>('buttons')

  const isDirty = useMemo(() => JSON.stringify(theme) !== JSON.stringify(savedTheme), [theme, savedTheme])

  const setColor = useCallback<PortalThemeContextValue['setColor']>((key, value) => {
    setTheme((prev) => ({ ...prev, colors: { ...prev.colors, [key]: value } }))
  }, [])

  const setFont = useCallback<PortalThemeContextValue['setFont']>((key, value) => {
    setTheme((prev) => ({ ...prev, fonts: { ...prev.fonts, [key]: value } }))
  }, [])

  const setBorder = useCallback<PortalThemeContextValue['setBorder']>((key, value) => {
    setTheme((prev) => ({ ...prev, borders: { ...prev.borders, [key]: value } }))
  }, [])

  const setSpacing = useCallback<PortalThemeContextValue['setSpacing']>((key, value) => {
    setTheme((prev) => ({ ...prev, spacing: { ...prev.spacing, [key]: value } }))
  }, [])

  const discard = useCallback(() => setTheme(savedTheme), [savedTheme])

  const markSaved = useCallback((saved: PortalThemeConfig) => {
    setSavedTheme(saved)
    setTheme(saved)
  }, [])

  const value: PortalThemeContextValue = {
    theme,
    savedTheme,
    isDirty,
    activeTab,
    setActiveTab,
    setColor,
    setFont,
    setBorder,
    setSpacing,
    discard,
    markSaved,
  }

  return <PortalThemeContext.Provider value={value}>{children}</PortalThemeContext.Provider>
}

export function usePortalThemeContext() {
  const ctx = useContext(PortalThemeContext)
  if (!ctx) throw new Error('usePortalThemeContext must be used inside PortalThemeProvider')
  return ctx
}

export { defaultTheme }
