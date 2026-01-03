import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'

// Types matching the backend abyss domain entities
export type ProviderType = 'oidc' | 'saml' | 'oauth2' | 'ldap'

export interface IdentityProvider {
  id: string
  realm_id: string
  alias: string
  display_name: string
  provider_type: ProviderType
  enabled: boolean
  config: Record<string, string>
  created_at: string
  updated_at: string
}

export interface CreateProviderInput {
  alias: string
  display_name: string
  provider_type: ProviderType
  enabled: boolean
  config: Record<string, string>
}

export interface UpdateProviderInput {
  display_name?: string
  enabled?: boolean
  config?: Record<string, string>
}

// Mock data for development
const mockProviders: IdentityProvider[] = [
  {
    id: '550e8400-e29b-41d4-a716-446655440001',
    realm_id: '550e8400-e29b-41d4-a716-446655440000',
    alias: 'google',
    display_name: 'Google',
    provider_type: 'oidc',
    enabled: true,
    config: {
      client_id: 'google-client-id',
      client_secret: '********',
      authorization_url: 'https://accounts.google.com/o/oauth2/v2/auth',
      token_url: 'https://oauth2.googleapis.com/token',
      userinfo_url: 'https://openidconnect.googleapis.com/v1/userinfo',
    },
    created_at: '2024-01-15T10:30:00Z',
    updated_at: '2024-01-15T10:30:00Z',
  },
  {
    id: '550e8400-e29b-41d4-a716-446655440002',
    realm_id: '550e8400-e29b-41d4-a716-446655440000',
    alias: 'discord',
    display_name: 'Discord',
    provider_type: 'oauth2',
    enabled: false,
    config: {
      client_id: 'discord-client-id',
      client_secret: '********',
      authorization_url: 'https://discord.com/api/oauth2/authorize',
      token_url: 'https://discord.com/api/oauth2/token',
      userinfo_url: 'https://discord.com/api/users/@me',
    },
    created_at: '2024-01-20T14:45:00Z',
    updated_at: '2024-02-01T09:15:00Z',
  },
]

// Simulate network delay
const delay = (ms: number) => new Promise((resolve) => setTimeout(resolve, ms))

// In-memory store for mock data
let providers = [...mockProviders]

// Mock API functions
async function fetchProviders(realmName: string): Promise<IdentityProvider[]> {
  await delay(300)
  // In real implementation, filter by realm
  console.log(`[Mock API] Fetching providers for realm: ${realmName}`)
  return providers
}

async function fetchProvider(
  realmName: string,
  providerId: string
): Promise<IdentityProvider> {
  await delay(200)
  console.log(
    `[Mock API] Fetching provider ${providerId} for realm: ${realmName}`
  )
  const provider = providers.find((p) => p.id === providerId)
  if (!provider) {
    throw new Error('Provider not found')
  }
  return provider
}

async function createProvider(
  realmName: string,
  input: CreateProviderInput
): Promise<IdentityProvider> {
  await delay(400)
  console.log(`[Mock API] Creating provider for realm: ${realmName}`, input)
  const newProvider: IdentityProvider = {
    id: crypto.randomUUID(),
    realm_id: '550e8400-e29b-41d4-a716-446655440000',
    alias: input.alias,
    display_name: input.display_name,
    provider_type: input.provider_type,
    enabled: input.enabled,
    config: input.config,
    created_at: new Date().toISOString(),
    updated_at: new Date().toISOString(),
  }
  providers = [...providers, newProvider]
  return newProvider
}

async function updateProvider(
  realmName: string,
  providerId: string,
  input: UpdateProviderInput
): Promise<IdentityProvider> {
  await delay(300)
  console.log(
    `[Mock API] Updating provider ${providerId} for realm: ${realmName}`,
    input
  )
  const index = providers.findIndex((p) => p.id === providerId)
  if (index === -1) {
    throw new Error('Provider not found')
  }
  const updated: IdentityProvider = {
    ...providers[index],
    ...(input.display_name && { display_name: input.display_name }),
    ...(input.enabled !== undefined && { enabled: input.enabled }),
    ...(input.config && { config: { ...providers[index].config, ...input.config } }),
    updated_at: new Date().toISOString(),
  }
  providers = providers.map((p) => (p.id === providerId ? updated : p))
  return updated
}

async function deleteProvider(
  realmName: string,
  providerId: string
): Promise<void> {
  await delay(300)
  console.log(
    `[Mock API] Deleting provider ${providerId} for realm: ${realmName}`
  )
  const index = providers.findIndex((p) => p.id === providerId)
  if (index === -1) {
    throw new Error('Provider not found')
  }
  providers = providers.filter((p) => p.id !== providerId)
}

async function toggleProvider(
  realmName: string,
  providerId: string,
  enabled: boolean
): Promise<IdentityProvider> {
  return updateProvider(realmName, providerId, { enabled })
}

// Query keys factory
export const identityProviderKeys = {
  all: ['identity-providers'] as const,
  lists: () => [...identityProviderKeys.all, 'list'] as const,
  list: (realm: string) => [...identityProviderKeys.lists(), realm] as const,
  details: () => [...identityProviderKeys.all, 'detail'] as const,
  detail: (realm: string, id: string) =>
    [...identityProviderKeys.details(), realm, id] as const,
}

// TanStack Query hooks
interface BaseQuery {
  realm: string
}

interface ProviderQuery extends BaseQuery {
  providerId: string
}

export const useIdentityProviders = ({ realm }: BaseQuery) => {
  return useQuery({
    queryKey: identityProviderKeys.list(realm),
    queryFn: () => fetchProviders(realm),
    enabled: !!realm,
  })
}

export const useIdentityProvider = ({ realm, providerId }: ProviderQuery) => {
  return useQuery({
    queryKey: identityProviderKeys.detail(realm, providerId),
    queryFn: () => fetchProvider(realm, providerId),
    enabled: !!realm && !!providerId,
  })
}

export const useCreateIdentityProvider = ({ realm }: BaseQuery) => {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: (input: CreateProviderInput) => createProvider(realm, input),
    onSuccess: () => {
      queryClient.invalidateQueries({
        queryKey: identityProviderKeys.list(realm),
      })
    },
  })
}

export const useUpdateIdentityProvider = ({
  realm,
  providerId,
}: ProviderQuery) => {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: (input: UpdateProviderInput) =>
      updateProvider(realm, providerId, input),
    onSuccess: (data) => {
      queryClient.setQueryData(
        identityProviderKeys.detail(realm, providerId),
        data
      )
      queryClient.invalidateQueries({
        queryKey: identityProviderKeys.list(realm),
      })
    },
  })
}

export const useDeleteIdentityProvider = ({
  realm,
  providerId,
}: ProviderQuery) => {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: () => deleteProvider(realm, providerId),
    onSuccess: () => {
      queryClient.invalidateQueries({
        queryKey: identityProviderKeys.list(realm),
      })
      queryClient.removeQueries({
        queryKey: identityProviderKeys.detail(realm, providerId),
      })
    },
  })
}

export const useToggleIdentityProvider = ({
  realm,
  providerId,
}: ProviderQuery) => {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: (enabled: boolean) =>
      toggleProvider(realm, providerId, enabled),
    onSuccess: (data) => {
      queryClient.setQueryData(
        identityProviderKeys.detail(realm, providerId),
        data
      )
      queryClient.invalidateQueries({
        queryKey: identityProviderKeys.list(realm),
      })
    },
  })
}
