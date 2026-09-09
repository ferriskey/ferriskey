import {
  Eye,
  LayoutGrid,
  Link2,
  Settings,
  Shield,
  UserCog,
  Users,
  type LucideIcon,
} from 'lucide-react'
import { Permissions } from '@/api/core.interface'

export interface PermissionEntry {
  key: string
  label: string
  description: string
}

export interface PermissionGroup {
  group: string
  icon: LucideIcon
  permissions: readonly PermissionEntry[]
}

export const permissionCatalogue: readonly PermissionGroup[] = [
  {
    group: 'User Management',
    icon: UserCog,
    permissions: [
      {
        key: Permissions.ManageUsers,
        label: 'Manage Users',
        description: 'Create, edit and delete accounts, and reset their credentials.',
      },
      {
        key: Permissions.ViewUsers,
        label: 'View Users',
        description: 'Read accounts and their attributes.',
      },
      {
        key: Permissions.QueryUsers,
        label: 'Query Users',
        description: 'List and search across accounts.',
      },
    ],
  },
  {
    group: 'Client Management',
    icon: LayoutGrid,
    permissions: [
      {
        key: Permissions.CreateClient,
        label: 'Create Client',
        description: 'Register a new client in the realm.',
      },
      {
        key: Permissions.ManageClients,
        label: 'Manage Clients',
        description: 'Edit and delete existing clients.',
      },
      {
        key: Permissions.ViewClients,
        label: 'View Clients',
        description: 'Read client configuration.',
      },
      {
        key: Permissions.QueryClients,
        label: 'Query Clients',
        description: 'List and search across clients.',
      },
    ],
  },
  {
    group: 'Role & Authorization',
    icon: Shield,
    permissions: [
      {
        key: Permissions.ManageRoles,
        label: 'Manage Roles',
        description: 'Create, edit and delete roles and their permissions.',
      },
      {
        key: Permissions.ViewRoles,
        label: 'View Roles',
        description: 'Read roles and what they grant.',
      },
      {
        key: Permissions.ManageAuthorization,
        label: 'Manage Authorization',
        description: 'Edit the authorization policies of the realm.',
      },
      {
        key: Permissions.ViewAuthorization,
        label: 'View Authorization',
        description: 'Read the authorization policies of the realm.',
      },
    ],
  },
  {
    group: 'Realm Management',
    icon: Settings,
    permissions: [
      {
        key: Permissions.ManageRealm,
        label: 'Manage Realm',
        description: 'Edit realm settings, tokens and login policy.',
      },
      {
        key: Permissions.ViewRealm,
        label: 'View Realm',
        description: 'Read realm settings.',
      },
      {
        key: Permissions.QueryRealms,
        label: 'Query Realms',
        description: 'List the realms the account can reach.',
      },
    ],
  },
  {
    group: 'Identity Providers',
    icon: Link2,
    permissions: [
      {
        key: Permissions.ManageIdentityProviders,
        label: 'Manage Identity Providers',
        description: 'Declare, edit and remove federated providers.',
      },
      {
        key: Permissions.ViewIdentityProviders,
        label: 'View Identity Providers',
        description: 'Read the configuration of federated providers.',
      },
    ],
  },
  {
    group: 'Events & Audit',
    icon: Eye,
    permissions: [
      {
        key: Permissions.ManageEvents,
        label: 'Manage Events',
        description: 'Configure event capture and retention.',
      },
      {
        key: Permissions.ViewEvents,
        label: 'View Events',
        description: 'Read the security event trail.',
      },
    ],
  },
  {
    group: 'Groups',
    icon: Users,
    permissions: [
      {
        key: Permissions.QueryGroups,
        label: 'Query Groups',
        description: 'List and search across groups.',
      },
    ],
  },
] as const

export const permissionCount = permissionCatalogue.reduce(
  (n, g) => n + g.permissions.length,
  0
)

export const allPermissionKeys = permissionCatalogue.flatMap((g) =>
  g.permissions.map((p) => p.key)
)

