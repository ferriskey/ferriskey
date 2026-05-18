import type { CSSProperties, FormEvent, ReactNode } from 'react'
import { useMemo } from 'react'
import { useParams } from 'react-router-dom'
import { useGetActivePortalTheme } from '@/api/portal-theme.api'
import { useGetPublicPortalLayout } from '@/api/portal-layouts.api'
import type { RouterParams } from '@/routes/router'
import type { BuilderNode } from '@/lib/builder-core'
import { generateBreakpointCss, treeToReactNode } from '@/lib/builder-portal'
import { mergeWithDefaults, themeToCssVars } from '@/pages/portal-theme/lib/theme'
import type { Schemas } from '@/api/api.client'
import { usePortalPageSubmit } from '../hooks/use-portal-page-submit'

function parseTree(tree: unknown): BuilderNode[] {
  if (Array.isArray(tree)) {
    return tree as BuilderNode[]
  }
  if (tree && typeof tree === 'object' && Array.isArray((tree as { children?: unknown }).children)) {
    return (tree as { children: BuilderNode[] }).children
  }
  return []
}

interface Props {
  children: ReactNode
  pageType: Schemas.PortalPageType
}

export function PortalLayoutWrapper({ children, pageType }: Props) {
  const { realm_name } = useParams<RouterParams>()
  const realm = realm_name ?? 'master'

  const { data: activeData, isLoading: isThemeLoading } = useGetActivePortalTheme({
    realm,
    pageType,
  })
  const layoutId = activeData?.layout_id
  const { data: layoutData, isLoading: isLayoutLoading } = useGetPublicPortalLayout({
    realm,
    layoutId: layoutId ?? '',
  })

  const cssVars = useMemo(
    () => themeToCssVars(mergeWithDefaults(activeData?.design_tokens)) as CSSProperties,
    [activeData?.design_tokens],
  )

  const pageTree = parseTree(activeData?.page_tree)
  const layoutTree = layoutData?.data ? parseTree(layoutData.data.tree) : []
  const { onSubmit } = usePortalPageSubmit(pageType)

  if (isThemeLoading || (layoutId && isLayoutLoading)) {
    return <div style={cssVars}>{children}</div>
  }

  // Page content: realm admin's custom tree when present, fall back to the
  // hardcoded React feature otherwise. When a custom tree is rendered, wrap
  // it in a <form> so submit_button can fire the page-specific handler.
  const handleSubmit = (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault()
    if (!onSubmit) return
    onSubmit(new FormData(event.currentTarget))
  }

  // Collect responsive overrides from both the page tree and the layout tree
  // (each node's id is unique across them) into a single <style> block.
  const breakpointCss = [
    generateBreakpointCss(pageTree),
    generateBreakpointCss(layoutTree),
  ]
    .filter(Boolean)
    .join('\n')

  const pageContent: ReactNode =
    pageTree.length > 0 ? (
      <form onSubmit={handleSubmit}>
        {treeToReactNode(pageTree, { runtime: true })}
      </form>
    ) : (
      <>{children}</>
    )

  const responsiveStyle = breakpointCss ? (
    <style dangerouslySetInnerHTML={{ __html: breakpointCss }} />
  ) : null

  if (layoutTree.length === 0) {
    return (
      <div style={cssVars}>
        {responsiveStyle}
        {pageContent}
      </div>
    )
  }

  return (
    <div style={cssVars}>
      {responsiveStyle}
      {treeToReactNode(layoutTree, { runtime: true, pageContent })}
    </div>
  )
}
