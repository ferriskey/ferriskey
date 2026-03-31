import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { toast } from 'sonner'
import { BaseQuery } from '.'

export const useGetOrganizations = ({ realm }: BaseQuery) => {
  return useQuery(
    window.tanstackApi.get('/realms/{realm_name}/organizations', {
      path: { realm_name: realm ?? 'master' },
    }).queryOptions
  )
}

export const useGetOrganization = ({
  realm,
  organizationId,
}: BaseQuery & { organizationId?: string }) => {
  return useQuery({
    ...window.tanstackApi.get('/realms/{realm_name}/organizations/{organization_id}', {
      path: { realm_name: realm!, organization_id: organizationId! },
    }).queryOptions,
    enabled: !!realm && !!organizationId,
  })
}

export const useCreateOrganization = () => {
  const queryClient = useQueryClient()
  return useMutation({
    ...window.tanstackApi.mutation('post', '/realms/{realm_name}/organizations').mutationOptions,
    onSuccess: async (payload, variables) => {
      const keys = window.tanstackApi.get('/realms/{realm_name}/organizations', {
        path: { realm_name: variables.path.realm_name },
      }).queryKey
      toast.success(`Organization "${payload.name}" created successfully`)
      await queryClient.invalidateQueries({ queryKey: keys })
    },
  })
}

export const useUpdateOrganization = () => {
  const queryClient = useQueryClient()
  return useMutation({
    ...window.tanstackApi.mutation('put', '/realms/{realm_name}/organizations/{organization_id}')
      .mutationOptions,
    onSuccess: async (payload, variables) => {
      const keys = window.tanstackApi.get(
        '/realms/{realm_name}/organizations/{organization_id}',
        {
          path: {
            realm_name: variables.path.realm_name,
            organization_id: variables.path.organization_id,
          },
        }
      ).queryKey
      toast.success(`Organization "${payload.name}" updated successfully`)
      queryClient.invalidateQueries({ queryKey: keys })
    },
  })
}

export const useDeleteOrganization = () => {
  const queryClient = useQueryClient()
  return useMutation({
    ...window.tanstackApi.mutation(
      'delete',
      '/realms/{realm_name}/organizations/{organization_id}'
    ).mutationOptions,
    onSuccess: async (_, variables) => {
      const keys = window.tanstackApi.get('/realms/{realm_name}/organizations', {
        path: { realm_name: variables.path.realm_name },
      }).queryKey
      toast.success('Organization deleted')
      await queryClient.invalidateQueries({ queryKey: keys })
    },
  })
}

export const useGetOrganizationAttributes = ({
  realm,
  organizationId,
}: BaseQuery & { organizationId?: string }) => {
  return useQuery({
    ...window.tanstackApi.get(
      '/realms/{realm_name}/organizations/{organization_id}/attributes',
      {
        path: { realm_name: realm!, organization_id: organizationId! },
      }
    ).queryOptions,
    select: (response) => response.data,
    enabled: !!realm && !!organizationId,
  })
}

export const useUpsertOrganizationAttribute = () => {
  const queryClient = useQueryClient()
  return useMutation({
    ...window.tanstackApi.mutation(
      'put',
      '/realms/{realm_name}/organizations/{organization_id}/attributes/{key}'
    ).mutationOptions,
    onSuccess: async (_, variables) => {
      const keys = window.tanstackApi.get(
        '/realms/{realm_name}/organizations/{organization_id}/attributes',
        {
          path: {
            realm_name: variables.path.realm_name,
            organization_id: variables.path.organization_id,
          },
        }
      ).queryKey
      await queryClient.invalidateQueries({ queryKey: keys })
    },
  })
}

export const useDeleteOrganizationAttribute = () => {
  const queryClient = useQueryClient()
  return useMutation({
    ...window.tanstackApi.mutation(
      'delete',
      '/realms/{realm_name}/organizations/{organization_id}/attributes/{key}'
    ).mutationOptions,
    onSuccess: async (_, variables) => {
      const keys = window.tanstackApi.get(
        '/realms/{realm_name}/organizations/{organization_id}/attributes',
        {
          path: {
            realm_name: variables.path.realm_name,
            organization_id: variables.path.organization_id,
          },
        }
      ).queryKey
      toast.success('Attribute deleted')
      await queryClient.invalidateQueries({ queryKey: keys })
    },
  })
}
