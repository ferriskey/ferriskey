type RoleLike = { name: string; client?: { client_id: string } | null }

export const selectedClientQuery = (value: string) => ({ client_id_exact: value, limit: 1 })

export const selectedRoleQuery = (value: string) => ({ qualified_name: value, limit: 2 })

export const roleValue = (role: RoleLike) => {
  const clientId = role.client?.client_id
  return clientId ? `${clientId}.${role.name}` : role.name
}

export const pickSelectedRole = <R extends RoleLike>(roles: R[] | undefined, value: string) =>
  roles?.find((candidate) => roleValue(candidate) === value)
