import type { CSSProperties, FormEvent, ReactNode } from 'react'
import { useMemo } from 'react'
import { useParams } from 'react-router-dom'
import {
  useGetActivePortalTheme,
  useGetPortalPageRequirements,
} from '@/api/portal-theme.api'
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

function collectTypes(nodes: BuilderNode[], acc: Set<string>) {
  for (const node of nodes) {
    if (node && typeof node.type === 'string') acc.add(node.type)
    if (Array.isArray(node?.children)) collectTypes(node.children, acc)
  }
}

function hasAllRequiredBlocks(tree: BuilderNode[], required: string[]): boolean {
  if (required.length === 0) return true
  const present = new Set<string>()
  collectTypes(tree, present)
  return required.every((type) => present.has(type))
}

// A layout without a `page-content` slot has nowhere to render the page,
// so it would produce a blank screen — treat it the same as "no layout"
// and fall back to the default hardcoded page.
function layoutHasPageContent(tree: BuilderNode[]): boolean {
  const present = new Set<string>()
  collectTypes(tree, present)
  return present.has('page-content')
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
  const { data: requirementsData, isLoading: isRequirementsLoading } =
    useGetPortalPageRequirements({ realm })

  const cssVars = useMemo(
    () => themeToCssVars(mergeWithDefaults(activeData?.design_tokens)) as CSSProperties,
    [activeData?.design_tokens],
  )

  const pageTree = parseTree(activeData?.page_tree)
  const layoutTree = layoutData?.data ? parseTree(layoutData.data.tree) : []
  const requiredBlocks = useMemo(() => {
    const entry = requirementsData?.data?.find((r) => r.page_type === pageType)
    return entry?.required_blocks ?? []
  }, [requirementsData, pageType])
  // The layout only wraps the custom page when that page is valid (i.e. ships
  // all required blocks). Otherwise we fall back to the hardcoded React
  // feature, which already provides its own layout — wrapping it inside the
  // admin's layout would render an incoherent mix of two designs.
  const pageIsValid = pageTree.length > 0 && hasAllRequiredBlocks(pageTree, requiredBlocks)
  const { onSubmit } = usePortalPageSubmit(pageType)

  if (
    isThemeLoading ||
    (layoutId && isLayoutLoading) ||
    isRequirementsLoading
  ) {
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

  const responsiveStyle = breakpointCss ? (
    <style dangerouslySetInnerHTML={{ __html: breakpointCss }} />
  ) : null

  // The layout is only renderable when it has a `page-content` slot.
  // Without one, applying it would hide the page entirely.
  const layoutIsValid = layoutTree.length > 0 && layoutHasPageContent(layoutTree)

  if (!pageIsValid || (layoutTree.length > 0 && !layoutIsValid)) {
    return (
      <div style={cssVars}>
        {responsiveStyle}
        {children}
      </div>
    )
  }

  const pageContent: ReactNode = (
    <form onSubmit={handleSubmit}>
      {treeToReactNode(pageTree, { runtime: true })}
    </form>
  )

  if (!layoutIsValid) {
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
