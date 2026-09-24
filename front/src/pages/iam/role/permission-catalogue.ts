import {
  Building2,
  Eye,
  KeyRound,
  LayoutGrid,
  Link2,
  Mail,
  Settings,
  Shield,
  UserCog,
  Users,
  Webhook,
  type LucideIcon,
} from 'lucide-react'
import { Permissions } from '@/api/core.interface'

export interface PermissionGroup {
  key: string
  icon: LucideIcon
  permissions: readonly string[]
}

export const permissionGroupLabelKey = (group: string) => `permissions.groups.${group}`

export const permissionLabelKey = (permission: string) =>
  `permissions.entries.${permission}.label`

export const permissionDescriptionKey = (permission: string) =>
  `permissions.entries.${permission}.description`

export const permissionCatalogue: readonly PermissionGroup[] = [
  {
    key: 'user_management',
    icon: UserCog,
    permissions: [Permissions.ManageUsers, Permissions.ViewUsers, Permissions.QueryUsers],
  },
  {
    key: 'client_management',
    icon: LayoutGrid,
    permissions: [
      Permissions.CreateClient,
      Permissions.ManageClients,
      Permissions.ViewClients,
      Permissions.QueryClients,
    ],
  },
  {
    key: 'role_authorization',
    icon: Shield,
    permissions: [
      Permissions.ManageRoles,
      Permissions.ViewRoles,
      Permissions.ManageAuthorization,
      Permissions.ViewAuthorization,
    ],
  },
  {
    key: 'realm_management',
    icon: Settings,
    permissions: [Permissions.ManageRealm, Permissions.ViewRealm, Permissions.QueryRealms],
  },
  {
    key: 'identity_providers',
    icon: Link2,
    permissions: [Permissions.ManageIdentityProviders, Permissions.ViewIdentityProviders],
  },
  {
    key: 'events_audit',
    icon: Eye,
    permissions: [Permissions.ManageEvents, Permissions.ViewEvents],
  },
  {
    key: 'groups',
    icon: Users,
    permissions: [Permissions.QueryGroups],
  },
  {
    key: 'client_scopes',
    icon: KeyRound,
    permissions: [
      Permissions.ManageClientScopes,
      Permissions.ViewClientScopes,
      Permissions.QueryClientScopes,
    ],
  },
  {
    key: 'webhooks',
    icon: Webhook,
    permissions: [
      Permissions.ManageWebhooks,
      Permissions.ViewWebhooks,
      Permissions.QueryWebhooks,
    ],
  },
  {
    key: 'email_templates',
    icon: Mail,
    permissions: [Permissions.ManageEmailTemplates, Permissions.ViewEmailTemplates],
  },
  {
    key: 'organizations',
    icon: Building2,
    permissions: [Permissions.ManageOrganizations, Permissions.ViewOrganizations],
  },
] as const

export const permissionCount = permissionCatalogue.reduce(
  (n, g) => n + g.permissions.length,
  0
)

export const allPermissionKeys = permissionCatalogue.flatMap((g) => [...g.permissions])
