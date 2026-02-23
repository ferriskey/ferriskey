import { RouterParams } from '@/routes/router'
import { useNavigate, useParams } from 'react-router'
import { useDeleteRole, useGetRoles } from '../../../api/role.api'
import PageRolesOverview from '../ui/page-roles-overview'
import { ROLE_CREATE_URL, ROLE_SETTINGS_URL, ROLE_URL, ROLES_URL } from '@/routes/sub-router/role.router'
import { Schemas } from '@/api/api.client'
import { useMemo, useState, useEffect } from 'react'
import { Filter, FilterFieldsConfig } from '@/components/ui/filters'
import { useConfirmDeleteAlert } from '@/hooks/use-confirm-delete-alert'
import { useOutletContext } from 'react-router'
import { RolesLayoutContext } from '../layout/roles-layout'

import Role = Schemas.Role

export default function PageRolesOverviewFeature() {
  const { realm_name } = useParams<RouterParams>()
  const navigate = useNavigate()
  const { data: rolesResponse, isLoading } = useGetRoles({ realm: realm_name ?? 'master' })
  const { mutate: deleteRole } = useDeleteRole()
  const { confirm, ask, close } = useConfirmDeleteAlert()
  const { setPrimaryAction } = useOutletContext<RolesLayoutContext>()

  useEffect(() => {
    setPrimaryAction({ label: 'New Role', onClick: () => navigate(`${ROLES_URL(realm_name)}${ROLE_CREATE_URL}`) })
    return () => setPrimaryAction(undefined)
  }, [])
  const [filters, setFilters] = useState<Filter[]>([])

  const roles = useMemo(() => rolesResponse?.data || [], [rolesResponse])

  // Calculate statistics
  const statistics = useMemo(() => {
    const totalRoles = roles.length
    const realmRoles = roles.filter(role => !role.client_id).length
    const clientRoles = roles.filter(role => role.client_id).length
    const rolesWithPermissions = roles.filter(role => role.permissions && role.permissions.length > 0).length

    return {
      totalRoles,
      realmRoles,
      clientRoles,
      rolesWithPermissions,
    }
  }, [roles])

  // Filter configuration
  const filterFields: FilterFieldsConfig = []

  const handleDeleteSelected = (items: Role[]) => {
    if (!realm_name) return
    items.forEach((role) => {
      deleteRole({
        path: {
          realm_name: realm_name ?? 'master',
          role_id: role.id,
        },
      })
    })
  }

  const handleClickRow = (roleId: string) => {
    navigate(`${ROLE_URL(realm_name, roleId)}${ROLE_SETTINGS_URL}`)
  }

  const handleCreateRole = () => {
    navigate(`${ROLES_URL(realm_name)}${ROLE_CREATE_URL}`)
  }

  const onRowDelete = (role: Role) => {
    ask({
      title: 'Delete role?',
      description: `Are you sure you want to delete "${role.name}"? This action cannot be undone.`,
      onConfirm: () => {
        if (!realm_name) return
        deleteRole({
          path: {
            realm_name: realm_name ?? 'master',
            role_id: role.id,
          },
        })
        close()
      },
    })
  }

  const handleFiltersChange = (newFilters: Filter[]) => {
    setFilters(newFilters)
  }

  return (
    <PageRolesOverview
      data={roles}
      isLoading={isLoading}
      realmName={realm_name ?? 'master'}
      statistics={statistics}
      filters={filters}
      filterFields={filterFields}
      onFiltersChange={handleFiltersChange}
      confirm={confirm}
      onConfirmClose={close}
      handleDeleteSelected={handleDeleteSelected}
      handleClickRow={handleClickRow}
      handleCreateRole={handleCreateRole}
      onRowDelete={onRowDelete}
    />
  )
}
