import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { toast } from 'sonner'
import { BaseQuery } from '.'
import type { Endpoints } from './api.client'
import { apiErrorMessage } from '@/lib/api-error'
import { translate } from '@/lib/i18n'

export type EmailTemplatesQuery = NonNullable<
  Endpoints.get_Fetch_templates['parameters']['query']
>

export type EmailTemplatesFilter = Omit<
  EmailTemplatesQuery,
  'page' | 'limit' | 'order' | 'order_by'
>

export const EMAIL_TEMPLATE_FILTER_KEYS = [
  'search',
  'name',
  'email_type',
  'created_from',
  'created_to',
] as const

export const EMAIL_TEMPLATE_SEARCH_LIMIT = 20

export const emailTemplatesKey = (realm: string) =>
  window.tanstackApi.get('/realms/{realm_name}/email-templates', {
    path: { realm_name: realm },
    query: {},
  }).queryKey

export const useGetEmailTemplates = ({
  realm = 'master',
  query,
  enabled = true,
}: BaseQuery & { query?: EmailTemplatesQuery; enabled?: boolean }) => {
  return useQuery({
    ...window.tanstackApi.get('/realms/{realm_name}/email-templates', {
      path: { realm_name: realm },
      query: query ?? {},
    }).queryOptions,
    enabled: enabled && !!realm,
  })
}

export const useEmailTemplateCount = ({
  realm = 'master',
  filter,
}: BaseQuery & { filter?: EmailTemplatesFilter }) => {
  const { data, isLoading } = useGetEmailTemplates({ realm, query: { ...filter, limit: 1 } })
  return { count: data?.metadata.total ?? 0, isLoading }
}

export const useGetEmailTemplate = ({ realm = 'master', templateId }: BaseQuery & { templateId: string }) => {
  return useQuery({
    ...window.tanstackApi.get('/realms/{realm_name}/email-templates/{template_id}', {
      path: { realm_name: realm, template_id: templateId },
    }).queryOptions,
    enabled: !!templateId && templateId !== 'new',
  })
}

export const useGetTemplateVariables = (emailType: string) => {
  return useQuery({
    ...window.tanstackApi.get('/email-templates/variables/{email_type}', {
      path: { email_type: emailType },
    }).queryOptions,
    enabled: !!emailType,
  })
}

export const useCreateEmailTemplate = () => {
  const queryClient = useQueryClient()
  return useMutation({
    ...window.tanstackApi.mutation('post', '/realms/{realm_name}/email-templates').mutationOptions,
    onSuccess: async (_, variables) => {
      const keys = emailTemplatesKey(variables.path.realm_name)

      await queryClient.invalidateQueries({ queryKey: keys })
      toast.success(translate('common:toast.email_template.created'))
    },
  })
}

export const useUpdateEmailTemplate = () => {
  const queryClient = useQueryClient()
  return useMutation({
    ...window.tanstackApi.mutation('put', '/realms/{realm_name}/email-templates/{template_id}').mutationOptions,
    onSuccess: async (_, variables) => {
      const keys = emailTemplatesKey(variables.path.realm_name)

      await queryClient.invalidateQueries({ queryKey: keys })
      toast.success(translate('common:toast.email_template.saved'))
    },
  })
}

export const useImportEmailTemplate = () => {
  const queryClient = useQueryClient()
  return useMutation({
    ...window.tanstackApi.mutation('post', '/realms/{realm_name}/email-templates/import')
      .mutationOptions,
    onSuccess: async (_, variables) => {
      const keys = emailTemplatesKey(variables.path.realm_name)

      await queryClient.invalidateQueries({ queryKey: keys })
      toast.success(translate('common:toast.email_template.imported'))
    },
    // The server refuses a file that belongs to the other builder or that uses
    // a format version it cannot read; its message says which, so surface it.
    onError: (error) => {
      toast.error(translate('common:toast.import_failed'), {
        description: apiErrorMessage(error),
      })
    },
  })
}

export const useDeleteEmailTemplate = () => {
  const queryClient = useQueryClient()
  return useMutation({
    ...window.tanstackApi.mutation('delete', '/realms/{realm_name}/email-templates/{template_id}').mutationOptions,
    onSuccess: async (_, variables) => {
      const keys = emailTemplatesKey(variables.path.realm_name)

      await queryClient.invalidateQueries({ queryKey: keys })
      toast.success(translate('common:toast.email_template.deleted'))
    },
  })
}
