import { useLocation, useParams } from 'react-router-dom'
import { CONSOLE_URL, REALM_URL, RouterParams } from '@/routes/router'

export const inConsole = (pathname: string) => pathname.includes('/console/')

export function useSectionBase(adminSegment: string, consoleSegment: string) {
  const { pathname } = useLocation()
  const { realm_name = 'master' } = useParams<RouterParams>()

  return inConsole(pathname)
    ? `${CONSOLE_URL(realm_name)}/${consoleSegment}`
    : `${REALM_URL(realm_name)}/${adminSegment}`
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
