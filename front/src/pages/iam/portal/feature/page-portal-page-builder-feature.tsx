import { useCallback, useMemo, useState, type CSSProperties } from 'react'
import { useNavigate, useParams } from 'react-router-dom'
import { useTranslation } from 'react-i18next'
import { toast } from 'sonner'
import { BasicSpinner } from '@/components/ui/spinner'
import {
  describePortalPageError,
  useGetPortalThemeById,
  useUpdatePortalThemePage,
} from '@/api/portal-theme.api'
import { useGetPortalLayouts } from '@/api/portal-layouts.api'
import { mergeWithDefaults, themeToCssVars } from '@/lib/portal-theme/theme'
import type { BuilderNode } from '@/lib/builder-core'
import { RouterParams } from '@/routes/router'
import { PORTAL_PAGES, type PortalPageType } from '../portal-pages'
import { usePortalUrls } from '../use-portal-urls'
import { parseTree, readPageTree } from '../theme-validation'
import PagePortalPageBuilder from '../ui/page-portal-page-builder'

const PAGE_TYPES = new Set<string>(PORTAL_PAGES.map((p) => p.type))

export default function PagePortalPageBuilderFeature() {
  const { t } = useTranslation('portal')
  const portal = usePortalUrls()
  const { realm_name, theme_id, page_type } = useParams<
    RouterParams & { theme_id: string; page_type: string }
  >()
  const navigate = useNavigate()
  const realm = realm_name ?? 'master'
  const themeId = theme_id ?? ''
  const pageType = (PAGE_TYPES.has(page_type ?? '') ? page_type : 'login') as PortalPageType

  const { data: themeData, isLoading } = useGetPortalThemeById({ realm, themeId })
  const { data: layoutsData } = useGetPortalLayouts({ realm })
  const { mutate: updatePage, isPending: isSaving } = useUpdatePortalThemePage()

  const theme = themeData?.data
  const layout = (layoutsData?.data ?? []).find((l) => l.id === theme?.layout_id)

  const cssVars = useMemo<CSSProperties>(
    () => themeToCssVars(mergeWithDefaults(theme?.config)) as CSSProperties,
    [theme]
  )

  const layoutTree = useMemo<BuilderNode[] | null>(
    () => (layout ? parseTree(layout.tree) : null),
    [layout]
  )

  const initialTree = useMemo(() => readPageTree(theme, pageType), [theme, pageType])

  const [draft, setDraft] = useState<BuilderNode[] | null>(null)
  const handleTreeChange = useCallback((tree: BuilderNode[]) => setDraft(tree), [])

  const handleSave = () => {
    if (!draft) return
    updatePage(
      {
        path: { realm_name: realm, theme_id: themeId, page_type: pageType },
        body: { tree: draft },
      },
      {
        onSuccess: () => {
          setDraft(null)
          toast.success(t('page_builder.toast.saved'))
        },
        onError: (error) => {
          toast.error(t('page_builder.toast.save_failed'), {
            description: describePortalPageError(error) ?? t('page_builder.toast.unknown_error'),
          })
        },
      }
    )
  }

  if (isLoading || !theme) {
    return (
      <div className='flex h-[60vh] items-center justify-center text-neutral-400 dark:text-neutral-500'>
        <BasicSpinner />
      </div>
    )
  }

  return (
    <PagePortalPageBuilder
      key={`${theme.id}-${pageType}`}
      realm={realm}
      themeName={theme.name}
      pageType={pageType}
      initialTree={initialTree}
      layoutTree={layoutTree}
      layoutName={layout?.name}
      cssVars={cssVars}
      isDirty={draft !== null}
      isSaving={isSaving}
      pageHref={(next) => portal.themePage(themeId, next)}
      onTreeChange={handleTreeChange}
      onBack={() => navigate(`${portal.theme(themeId)}/pages`)}
      onSave={handleSave}
    />
  )
}
