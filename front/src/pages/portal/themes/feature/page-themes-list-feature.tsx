import { useLocation, useNavigate, useParams } from 'react-router-dom'
import type { RouterParams } from '@/routes/router'
import {
  useActivatePortalTheme,
  useCreatePortalTheme,
  useDeletePortalTheme,
  useGetActivePortalTheme,
  useListPortalThemes,
} from '@/api/portal-theme.api'
import { themeBuilderUrl } from '@/routes/sub-router/portal-theme.router'
import PageThemesList from '../ui/page-themes-list'
import { defaultTheme } from '@/pages/portal-theme/lib/theme'

export default function PageThemesListFeature() {
  const { realm_name } = useParams<RouterParams>()
  const navigate = useNavigate()
  const { pathname } = useLocation()
  const realm = realm_name ?? 'master'

  const { data: listData, isLoading } = useListPortalThemes({ realm })
  const { data: activeData } = useGetActivePortalTheme({ realm, pageType: 'login' })
  const activeThemeId = activeData?.theme_id ?? null

  const { mutate: createTheme, isPending: isCreating } = useCreatePortalTheme()
  const { mutate: deleteTheme } = useDeletePortalTheme()
  const { mutate: activateTheme } = useActivatePortalTheme()

  const handleCreate = (name: string) => {
    createTheme(
      {
        path: { realm_name: realm },
        body: { name, config: defaultTheme },
      },
      {
        onSuccess: (res) => {
          const newId = res?.data?.id
          if (newId) {
            navigate(themeBuilderUrl(pathname, realm, newId))
          }
        },
      },
    )
  }

  const handleEdit = (themeId: string) => {
    navigate(themeBuilderUrl(pathname, realm, themeId))
  }

  const handleActivate = (themeId: string) => {
    activateTheme({ path: { realm_name: realm, theme_id: themeId } })
  }

  const handleDelete = (themeId: string) => {
    deleteTheme({ path: { realm_name: realm, theme_id: themeId } })
  }

  return (
    <PageThemesList
      themes={listData?.data ?? []}
      activeThemeId={activeThemeId}
      isLoading={isLoading}
      isCreating={isCreating}
      onCreate={handleCreate}
      onEdit={handleEdit}
      onActivate={handleActivate}
      onDelete={handleDelete}
    />
  )
}
