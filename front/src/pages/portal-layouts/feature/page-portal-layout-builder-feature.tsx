import { useCallback, useMemo, useState, type CSSProperties } from 'react'
import { useNavigate, useParams } from 'react-router-dom'
import {
  useCreatePortalLayout,
  useGetPortalLayout,
  useUpdatePortalLayout,
} from '@/api/portal-layouts.api'
import { useGetPortalTheme } from '@/api/portal-theme.api'
import { BasicSpinner } from '@/components/ui/spinner'
import type { BuilderNode } from '@/lib/builder-core'
import { createPortalAdapter } from '@/lib/builder-portal'
import { defaultTheme, mergeWithDefaults, themeToCssVars } from '@/pages/portal-theme/lib/theme'
import {
  PORTAL_LAYOUT_BUILDER_URL,
  PORTAL_LAYOUTS_URL,
  type PortalLayoutRouterParams,
} from '@/routes/sub-router/portal-layouts.router'
import PagePortalLayoutBuilder from '../ui/page-portal-layout-builder'

// A new layout starts with an empty tree — the admin builds the whole thing
// from scratch. The `page-content` slot lives in the "Required for this
// layout" section of the component library if/when they want to add it.
function seedTree(): BuilderNode[] {
  return []
}

function parseTree(tree: unknown): BuilderNode[] {
  if (Array.isArray(tree)) {
    return tree as BuilderNode[]
  }
  if (tree && typeof tree === 'object' && Array.isArray((tree as { children?: unknown }).children)) {
    return (tree as { children: BuilderNode[] }).children
  }
  return []
}

export default function PagePortalLayoutBuilderFeature() {
  const { realm_name, layout_id } = useParams<PortalLayoutRouterParams>()
  const navigate = useNavigate()
  const realm = realm_name ?? 'master'
  const isNew = layout_id === 'new'

  const { data: layoutData, isLoading } = useGetPortalLayout({
    realm,
    layoutId: layout_id ?? '',
  })

  const { data: themeData } = useGetPortalTheme({ realm })

  if (!isNew && isLoading) {
    return (
      <div className='flex h-[60vh] items-center justify-center text-muted-foreground'>
        <BasicSpinner />
      </div>
    )
  }

  return (
    <BuilderFeatureInner
      key={layoutData?.data?.id ?? 'new'}
      realmName={realm_name}
      realm={realm}
      layoutId={layout_id ?? ''}
      isNew={isNew}
      initialName={layoutData?.data?.name ?? ''}
      initialTree={isNew ? seedTree() : parseTree(layoutData?.data?.tree)}
      themeConfig={themeData?.data}
      navigate={navigate}
    />
  )
}

function BuilderFeatureInner({
  realmName,
  realm,
  layoutId,
  isNew,
  initialName,
  initialTree,
  themeConfig,
  navigate,
}: {
  realmName: string | undefined
  realm: string
  layoutId: string
  isNew: boolean
  initialName: string
  initialTree: BuilderNode[]
  themeConfig: unknown
  navigate: ReturnType<typeof useNavigate>
}) {
  const [name, setName] = useState(initialName)
  const [tree, setTree] = useState<BuilderNode[]>(initialTree)

  const adapter = useMemo(() => createPortalAdapter(), [])

  const cssVars = useMemo<CSSProperties>(() => {
    const merged = themeConfig
      ? mergeWithDefaults(themeConfig as Parameters<typeof mergeWithDefaults>[0])
      : defaultTheme
    return themeToCssVars(merged) as CSSProperties
  }, [themeConfig])

  const { mutate: createLayout, isPending: isCreating } = useCreatePortalLayout()
  const { mutate: updateLayout, isPending: isUpdating } = useUpdatePortalLayout()

  const handleTreeChange = useCallback((newTree: BuilderNode[]) => {
    setTree(newTree)
  }, [])

  const handleSave = () => {
    if (isNew) {
      createLayout(
        {
          path: { realm_name: realm },
          body: { name, tree: tree as unknown as Record<string, unknown> },
        },
        {
          onSuccess: (res) => {
            const newId = res?.data?.id
            if (newId) {
              navigate(PORTAL_LAYOUT_BUILDER_URL(realmName, newId), { replace: true })
            }
          },
        },
      )
    } else {
      updateLayout({
        path: { realm_name: realm, layout_id: layoutId },
        body: { name, tree: tree as unknown as Record<string, unknown> },
      })
    }
  }

  const handleBack = () => {
    navigate(PORTAL_LAYOUTS_URL(realmName))
  }

  return (
    <PagePortalLayoutBuilder
      adapter={adapter}
      tree={tree}
      onTreeChange={handleTreeChange}
      name={name}
      onNameChange={setName}
      isNew={isNew}
      isSaving={isCreating || isUpdating}
      cssVars={cssVars}
      onSave={handleSave}
      onBack={handleBack}
    />
  )
}
