import { useEffect, useState } from 'react'

type Theme = 'light' | 'dark' | 'system'

export function useTheme() {
  const [theme, setTheme] = useState<Theme>(() => {
    if (typeof window !== 'undefined') {
      return (localStorage.getItem('theme') as Theme) || 'system'
    }
    return 'system'
  })

  useEffect(() => {
    const root = window.document.documentElement
    const body = window.document.body

    root.classList.remove('light', 'dark')
    body.classList.remove('light-mode', 'dark-mode')

    if (theme === 'system') {
      const systemTheme = window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light'
      root.classList.add(systemTheme)

      const systemThemeBody = window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark-mode' : 'light-mode'
      body.classList.add(systemThemeBody)
      return
    }

    root.classList.add(theme)
    body.classList.add(theme === 'dark' ? 'dark-mode' : 'light-mode')
  }, [theme])

  const setThemeMode = (newTheme: Theme) => {
    setTheme(newTheme)
    localStorage.setItem('theme', newTheme)
  }

  return { theme, setTheme: setThemeMode }
} 