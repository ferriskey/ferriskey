import type { CSSProperties, ReactNode } from 'react'
import { useMemo } from 'react'
import { useParams } from 'react-router-dom'
import { useGetActivePortalTheme } from '@/api/portal-theme.api'
import { useGetPortalLayout } from '@/api/portal-layouts.api'
import type { RouterParams } from '@/routes/router'
import type { BuilderNode } from '@/lib/builder-core'
import { treeToReactNode } from '@/lib/builder-portal'
import { mergeWithDefaults, themeToCssVars } from '@/pages/portal-theme/lib/theme'
import type { Schemas } from '@/api/api.client'

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
  const { data: layoutData, isLoading: isLayoutLoading } = useGetPortalLayout({
    realm,
    layoutId: layoutId ?? '',
  })

  const cssVars = useMemo(
    () => themeToCssVars(mergeWithDefaults(activeData?.design_tokens)) as CSSProperties,
    [activeData?.design_tokens],
  )

  const layoutTree = layoutData?.data ? parseTree(layoutData.data.tree) : []

  if (isThemeLoading || (layoutId && isLayoutLoading)) {
    return <div style={cssVars}>{children}</div>
  }

  if (layoutTree.length === 0) {
    return <div style={cssVars}>{children}</div>
  }

  return <div style={cssVars}>{treeToReactNode(layoutTree, { pageContent: children })}</div>
}
