import type { ReactNode } from 'react'
import { useParams } from 'react-router-dom'
import { useGetPublicDefaultPortalLayout } from '@/api/portal-layouts.api'
import type { RouterParams } from '@/routes/router'
import type { BuilderNode } from '@/lib/builder-core'
import { treeToReactNode } from '@/lib/builder-portal'

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
}

export function PortalLayoutWrapper({ children }: Props) {
  const { realm_name } = useParams<RouterParams>()
  const { data, isLoading } = useGetPublicDefaultPortalLayout({
    realm: realm_name ?? 'master',
  })

  const layout = data?.data
  const tree = layout ? parseTree(layout.tree) : []

  if (isLoading || !layout || tree.length === 0) {
    return <>{children}</>
  }

  return <>{treeToReactNode(tree, { pageContent: children })}</>
}
