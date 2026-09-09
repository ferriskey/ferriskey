import { useLocation, useParams } from 'react-router-dom'
import { RouterParams } from '@/routes/router'
import { NEXT_URL } from '@/next/routes'

export const NEXT_CONSOLE_ROOT = (realmName: string) => `${NEXT_URL(realmName)}/console`

export const inConsole = (pathname: string) => pathname.includes('/next/console/')

export function useSectionBase(adminSegment: string, consoleSegment: string) {
  const { pathname } = useLocation()
  const { realm_name = 'master' } = useParams<RouterParams>()

  return inConsole(pathname)
    ? `${NEXT_CONSOLE_ROOT(realm_name)}/${consoleSegment}`
    : `${NEXT_URL(realm_name)}/${adminSegment}`
}

export const useEmailTemplatesBase = () =>
  useSectionBase('email-templates', 'branding/email-templates')
export const usePortalBase = () => useSectionBase('portal', 'branding')
export const useIdentityProvidersBase = () =>
  useSectionBase('identity-providers', 'authentication/identity-providers')
export const useUsersBase = () => useSectionBase('users', 'user-management/identities')
export const useRolesBase = () => useSectionBase('roles', 'user-management/roles')
export const useOrganizationsBase = () =>
  useSectionBase('organizations', 'user-management/organizations')
export const useClientsBase = () => useSectionBase('clients', 'applications')
