import { usePortalBase } from '@/hooks/use-section-base'

export function usePortalUrls() {
  const base = usePortalBase()

  const themes = () => `${base}/themes`
  const theme = (themeId: string) => `${themes()}/${themeId}`
  const themePage = (themeId: string, pageType: string) => `${theme(themeId)}/pages/${pageType}`
  const layouts = () => `${base}/layouts`
  const layout = (layoutId: string) => `${layouts()}/${layoutId}`

  return { base, themes, theme, themePage, layouts, layout }
}
