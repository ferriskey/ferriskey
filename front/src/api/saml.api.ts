import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import type { Schemas } from './api.client'

export const samlConfigQueryKey = (realmName: string, clientId: string) => [
  'saml-config',
  realmName,
  clientId,
]

export const samlAttributeMappersQueryKey = (realmName: string, clientId: string) => [
  'saml-attribute-mappers',
  realmName,
  clientId,
]

function isNotConfigured(error: unknown): boolean {
  return (
    typeof error === 'object' &&
    error !== null &&
    'status' in error &&
    (error as { status?: number }).status === 404
  )
}

export const useGetSamlConfig = ({
  realmName,
  clientId,
}: {
  realmName?: string
  clientId?: string
}) => {
  return useQuery({
    queryKey: samlConfigQueryKey(realmName ?? '', clientId ?? ''),
    queryFn: async (): Promise<Schemas.ClientSamlConfig | null> => {
      try {
        return (await window.tanstackApi.client.get(
          '/realms/{realm_name}/clients/{client_id}/saml-config',
          { path: { realm_name: realmName!, client_id: clientId! } }
        )) as Schemas.ClientSamlConfig
      } catch (error) {
        if (isNotConfigured(error)) return null
        throw error
      }
    },
    enabled: !!realmName && !!clientId,
  })
}

export interface UpsertSamlConfigMutate {
  realmName: string
  clientId: string
  payload: Schemas.SetClientSamlConfigValidator
}

export const useUpsertSamlConfig = () => {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: async ({ realmName, clientId, payload }: UpsertSamlConfigMutate) => {
      return window.tanstackApi.client.put(
        '/realms/{realm_name}/clients/{client_id}/saml-config',
        {
          path: { realm_name: realmName, client_id: clientId },
          body: payload,
        }
      ) as Promise<Schemas.ClientSamlConfig>
    },
    onSuccess: (_, variables) => {
      queryClient.invalidateQueries({
        queryKey: samlConfigQueryKey(variables.realmName, variables.clientId),
      })
    },
  })
}

export const useGetSamlAttributeMappers = ({
  realmName,
  clientId,
}: {
  realmName?: string
  clientId?: string
}) => {
  return useQuery({
    queryKey: samlAttributeMappersQueryKey(realmName ?? '', clientId ?? ''),
    queryFn: async () =>
      window.tanstackApi.client.get(
        '/realms/{realm_name}/clients/{client_id}/saml-attribute-mappers',
        { path: { realm_name: realmName!, client_id: clientId! } }
      ) as Promise<Schemas.SamlAttributeMapper[]>,
    enabled: !!realmName && !!clientId,
  })
}

export interface CreateSamlAttributeMapperMutate {
  realmName: string
  clientId: string
  payload: Schemas.CreateSamlAttributeMapperValidator
}

export const useCreateSamlAttributeMapper = () => {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: async ({ realmName, clientId, payload }: CreateSamlAttributeMapperMutate) => {
      return window.tanstackApi.client.post(
        '/realms/{realm_name}/clients/{client_id}/saml-attribute-mappers',
        {
          path: { realm_name: realmName, client_id: clientId },
          body: payload,
        }
      ) as Promise<Schemas.SamlAttributeMapper>
    },
    onSuccess: (_, variables) => {
      queryClient.invalidateQueries({
        queryKey: samlAttributeMappersQueryKey(variables.realmName, variables.clientId),
      })
    },
  })
}

export interface DeleteSamlAttributeMapperMutate {
  realmName: string
  clientId: string
  mapperId: string
}

export const useDeleteSamlAttributeMapper = () => {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: async ({ realmName, clientId, mapperId }: DeleteSamlAttributeMapperMutate) => {
      return window.tanstackApi.client.delete(
        '/realms/{realm_name}/clients/{client_id}/saml-attribute-mappers/{mapper_id}',
        {
          path: { realm_name: realmName, client_id: clientId, mapper_id: mapperId },
        }
      )
    },
    onSuccess: (_, variables) => {
      queryClient.invalidateQueries({
        queryKey: samlAttributeMappersQueryKey(variables.realmName, variables.clientId),
      })
    },
  })
}
