import { Permissions } from '@/api/core.interface.ts'

export const permissionGroups = {
  'User Management': [
    Permissions.ManageUsers,
    Permissions.ViewUsers,
    Permissions.QueryUsers,
  ],
  'Client Management': [
    Permissions.CreateClient,
    Permissions.ManageClients,
    Permissions.ViewClients,
    Permissions.QueryClients,
  ],
  'Role & Authorization': [
    Permissions.ManageRoles,
    Permissions.ViewRoles,
    Permissions.ManageAuthorization,
    Permissions.ViewAuthorization,
  ],
  'Realm Management': [
    Permissions.ManageRealm,
    Permissions.ViewRealm,
    Permissions.QueryRealms,
  ],
  'Identity Providers': [
    Permissions.ManageIdentityProviders,
    Permissions.ViewIdentityProviders,
  ],
  'Events & Audit': [
    Permissions.ManageEvents,
    Permissions.ViewEvents,
  ],
  'Groups': [
    Permissions.QueryGroups,
  ],
}
