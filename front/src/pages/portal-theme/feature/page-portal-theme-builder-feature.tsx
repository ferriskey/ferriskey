import { useParams } from 'react-router'
import { RouterParams } from '@/routes/router'
import { useGetPortalTheme, useUpdatePortalTheme } from '@/api/portal-theme.api'
import { BasicSpinner } from '@/components/ui/spinner'
import {
  PortalThemeProvider,
  usePortalThemeContext,
} from '../context/portal-theme-context'
import PagePortalThemeBuilder from '../ui/page-portal-theme-builder'

function BuilderInner({ realmName }: { realmName: string }) {
  const { theme, markSaved } = usePortalThemeContext()
  const { mutate, isPending } = useUpdatePortalTheme()

  const handleSave = () => {
    mutate(
      {
        path: { realm_name: realmName },
        body: { config: theme },
      },
      {
        onSuccess: () => markSaved(theme),
      },
    )
  }

  return <PagePortalThemeBuilder isSaving={isPending} onSave={handleSave} />
}

export default function PagePortalThemeBuilderFeature() {
  const { realm_name } = useParams<RouterParams>()
  const { data, isLoading } = useGetPortalTheme({ realm: realm_name })

  if (isLoading || !realm_name) {
    return (
      <div className='flex h-[60vh] items-center justify-center text-muted-foreground'>
        <BasicSpinner />
      </div>
    )
  }

  return (
    <PortalThemeProvider initial={data?.data}>
      <BuilderInner realmName={realm_name} />
    </PortalThemeProvider>
  )
}
