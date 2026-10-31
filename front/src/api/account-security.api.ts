import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import type { BaseQuery } from '.'
import type { Endpoints } from './api.client'
import { previousPagePlaceholder, type PagedQueryOptions } from './paged-query'

const credentialsPath = '/realms/{realm_name}/users/me/credentials'

export const useGetOwnCredentials = ({ realm }: BaseQuery) => {
  return useQuery({
    ...window.tanstackApi.get(credentialsPath, {
      path: { realm_name: realm || 'master' },
    }).queryOptions,
  })
}

const useInvalidateOwnCredentials = () => {
  const queryClient = useQueryClient()

  return (realm: string) => {
    const queryKey = window.tanstackApi.get(credentialsPath, {
      path: { realm_name: realm },
    }).queryKey

    return queryClient.invalidateQueries({ queryKey })
  }
}

export const useRequestElevation = () => {
  return useMutation({
    ...window.tanstackApi.mutation('post', '/realms/{realm_name}/users/me/reauthenticate')
      .mutationOptions,
  })
}

export const useChangeOwnPassword = () => {
  return useMutation({
    ...window.tanstackApi.mutation('put', '/realms/{realm_name}/users/me/password').mutationOptions,
  })
}

export const useStartOwnOtpEnrollment = () => {
  return useMutation({
    ...window.tanstackApi.mutation('post', '/realms/{realm_name}/users/me/mfa/otp').mutationOptions,
  })
}

export const useConfirmOwnOtpEnrollment = () => {
  const invalidate = useInvalidateOwnCredentials()

  return useMutation({
    ...window.tanstackApi.mutation('put', '/realms/{realm_name}/users/me/mfa/otp').mutationOptions,
    onSuccess: (_res, variables) => invalidate(variables.path.realm_name),
  })
}

export const useDisableOwnOtp = () => {
  const invalidate = useInvalidateOwnCredentials()

  return useMutation({
    ...window.tanstackApi.mutation('delete', '/realms/{realm_name}/users/me/mfa/otp')
      .mutationOptions,
    onSuccess: (_res, variables) => invalidate(variables.path.realm_name),
  })
}

export const useStartOwnPasskeyRegistration = () => {
  return useMutation({
    ...window.tanstackApi.mutation('post', '/realms/{realm_name}/users/me/passkeys/options')
      .mutationOptions,
  })
}

export const useConfirmOwnPasskeyRegistration = () => {
  const invalidate = useInvalidateOwnCredentials()

  return useMutation({
    ...window.tanstackApi.mutation('post', '/realms/{realm_name}/users/me/passkeys')
      .mutationOptions,
    onSuccess: (_res, variables) => invalidate(variables.path.realm_name),
  })
}

export const useDeleteOwnPasskey = () => {
  const invalidate = useInvalidateOwnCredentials()

  return useMutation({
    ...window.tanstackApi.mutation('delete', '/realms/{realm_name}/users/me/passkeys/{credential_id}')
      .mutationOptions,
    onSuccess: (_res, variables) => invalidate(variables.path.realm_name),
  })
}

const sessionsPath = '/realms/{realm_name}/users/me/sessions'

export type OwnSessionsQuery = NonNullable<Endpoints.get_List_own_sessions['parameters']['query']>

export const useGetOwnSessions = ({
  realm,
  query,
  keepPrevious = false,
}: BaseQuery & { query?: OwnSessionsQuery } & PagedQueryOptions) => {
  return useQuery({
    ...window.tanstackApi.get(sessionsPath, {
      path: { realm_name: realm! },
      query: query ?? {},
    }).queryOptions,
    placeholderData: previousPagePlaceholder(keepPrevious),
    enabled: !!realm,
  })
}

const useInvalidateOwnSessions = () => {
  const queryClient = useQueryClient()

  return (realm: string) => {
    const queryKey = window.tanstackApi.get(sessionsPath, {
      path: { realm_name: realm },
    }).queryKey

    return queryClient.invalidateQueries({ queryKey })
  }
}

export const useRevokeOwnSession = () => {
  const invalidate = useInvalidateOwnSessions()

  return useMutation({
    ...window.tanstackApi.mutation('delete', '/realms/{realm_name}/users/me/sessions/{session_id}')
      .mutationOptions,
    onSuccess: (_res, variables) => invalidate(variables.path.realm_name),
  })
}

export const useRevokeOtherOwnSessions = () => {
  const invalidate = useInvalidateOwnSessions()

  return useMutation({
    ...window.tanstackApi.mutation('post', '/realms/{realm_name}/users/me/sessions/revoke-others')
      .mutationOptions,
    onSuccess: (_res, variables) => invalidate(variables.path.realm_name),
  })
}
