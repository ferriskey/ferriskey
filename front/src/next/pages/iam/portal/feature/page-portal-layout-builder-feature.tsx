import { useCallback, useMemo, useState, type CSSProperties } from 'react'
import { useNavigate, useParams } from 'react-router-dom'
import {
  useCreatePortalLayout,
  useGetPortalLayout,
  useUpdatePortalLayout,
} from '@/api/portal-layouts.api'
import { useGetPortalTheme, useListPortalThemes } from '@/api/portal-theme.api'
import { BasicSpinner } from '@/components/ui/spinner'
import type { BuilderNode } from '@/lib/builder-core'
import { createPortalAdapter } from '@/lib/builder-portal'
import { defaultTheme, mergeWithDefaults, themeToCssVars } from '@/pages/portal-theme/lib/theme'
import { RouterParams } from '@/routes/router'
import { NEXT_PORTAL_LAYOUTS_URL, NEXT_PORTAL_LAYOUT_URL } from '../portal-urls'
import { parseTree } from '../theme-validation'
import PagePortalLayoutBuilder from '../ui/page-portal-layout-builder'

export default function PagePortalLayoutBuilderFeature() {
  const { realm_name, layout_id } = useParams<RouterParams & { layout_id: string }>()
  const navigate = useNavigate()
  const realm = realm_name ?? 'master'
  const layoutId = layout_id ?? ''
  const isNew = layoutId === 'new'

  const { data: layoutData, isLoading } = useGetPortalLayout({ realm, layoutId })
  const { data: themeData } = useGetPortalTheme({ realm })
  const { data: themesData } = useListPortalThemes({ realm })

  if (!isNew && isLoading) {
    return (
      <div className='flex h-[60vh] items-center justify-center text-neutral-400 dark:text-neutral-500'>
        <BasicSpinner />
      </div>
    )
  }

  const layout = layoutData?.data

  return (
    <LayoutBuilderInner
      key={layout?.id ?? 'new'}
      realm={realm}
      layoutId={layoutId}
      isNew={isNew}
      isDefault={layout?.is_default ?? false}
      usedBy={(themesData?.data ?? [])
        .filter((t) => layout && t.layout_id === layout.id)
        .map((t) => t.name)}
      initialName={layout?.name ?? ''}
      initialTree={isNew ? [] : parseTree(layout?.tree)}
      themeConfig={themeData?.data}
      onNavigate={navigate}
    />
  )
}

interface InnerProps {
  realm: string
  layoutId: string
  isNew: boolean
  isDefault: boolean
  usedBy: string[]
  initialName: string
  initialTree: BuilderNode[]
  themeConfig: Parameters<typeof mergeWithDefaults>[0]
  onNavigate: ReturnType<typeof useNavigate>
}

function LayoutBuilderInner({
  realm,
  layoutId,
  isNew,
  isDefault,
  usedBy,
  initialName,
  initialTree,
  themeConfig,
  onNavigate,
}: InnerProps) {
  const [name, setName] = useState(initialName)
  const [tree, setTree] = useState<BuilderNode[]>(initialTree)

  const adapter = useMemo(() => createPortalAdapter(), [])

  const cssVars = useMemo<CSSProperties>(() => {
    const merged = themeConfig ? mergeWithDefaults(themeConfig) : defaultTheme
    return themeToCssVars(merged) as CSSProperties
  }, [themeConfig])

  const { mutate: createLayout, isPending: isCreating } = useCreatePortalLayout()
  const { mutate: updateLayout, isPending: isUpdating } = useUpdatePortalLayout()

  const handleTreeChange = useCallback((next: BuilderNode[]) => setTree(next), [])

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
              onNavigate(NEXT_PORTAL_LAYOUT_URL(realm, newId), { replace: true })
            }
          },
        }
      )
      return
    }

    updateLayout({
      path: { realm_name: realm, layout_id: layoutId },
      body: { name, tree: tree as unknown as Record<string, unknown> },
    })
  }

  return (
    <PagePortalLayoutBuilder
      adapter={adapter}
      tree={tree}
      name={name}
      isNew={isNew}
      isSaving={isCreating || isUpdating}
      isDefault={isDefault}
      usedBy={usedBy}
      cssVars={cssVars}
      onTreeChange={handleTreeChange}
      onNameChange={setName}
      onSave={handleSave}
      onBack={() => onNavigate(NEXT_PORTAL_LAYOUTS_URL(realm))}
    />
  )
}
