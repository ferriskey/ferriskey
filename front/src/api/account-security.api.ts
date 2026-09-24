import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { BaseQuery } from '.'

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
