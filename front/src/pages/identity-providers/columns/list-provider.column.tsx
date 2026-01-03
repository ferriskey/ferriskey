import BadgeColor from '@/components/ui/badge-color'
import { BadgeColorScheme } from '@/components/ui/badge-color.enum'
import { ColumnDef } from '@/components/ui/data-table'
import type { IdentityProvider, ProviderType } from '@/api/identity-providers.api'

const providerTypeColors: Record<ProviderType, BadgeColorScheme> = {
  oidc: BadgeColorScheme.BLUE,
  oauth2: BadgeColorScheme.PURPLE,
  saml: BadgeColorScheme.INDIGO,
  ldap: BadgeColorScheme.GREEN,
}

const providerTypeLabels: Record<ProviderType, string> = {
  oidc: 'OIDC',
  oauth2: 'OAuth2',
  saml: 'SAML',
  ldap: 'LDAP',
}

export const columns: ColumnDef<IdentityProvider>[] = [
  {
    id: 'name',
    header: 'Provider',
    cell: (provider) => (
      <div className='flex items-center gap-3'>
        <div className='h-10 w-10 rounded-lg bg-gradient-to-br from-primary/20 to-primary/10 flex items-center justify-center border border-primary/20'>
          <span className='text-sm font-semibold text-primary'>
            {provider.display_name?.[0]?.toUpperCase() || 'P'}
          </span>
        </div>
        <div className='flex flex-col'>
          <div className='font-semibold text-sm'>{provider.display_name}</div>
          <div className='text-xs text-muted-foreground font-mono'>{provider.alias}</div>
        </div>
      </div>
    ),
  },
  {
    id: 'type',
    header: 'Type',
    cell: (provider) => (
      <BadgeColor color={providerTypeColors[provider.provider_type]}>
        {providerTypeLabels[provider.provider_type]}
      </BadgeColor>
    ),
  },
  {
    id: 'status',
    header: 'Status',
    cell: (provider) => (
      <div className='flex items-center gap-2'>
        <span
          className={`h-2 w-2 rounded-full ${provider.enabled ? 'bg-emerald-500' : 'bg-red-500'}`}
        ></span>
        <span className='text-sm'>{provider.enabled ? 'Enabled' : 'Disabled'}</span>
      </div>
    ),
  },
  {
    id: 'updated_at',
    header: 'Last Updated',
    cell: (provider) => (
      <span className='text-sm text-muted-foreground'>
        {new Date(provider.updated_at).toLocaleDateString()}
      </span>
    ),
  },
]
