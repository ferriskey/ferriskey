import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { toast } from 'sonner'
import { BaseQuery } from '.'

export const useGetEmailTemplates = ({ realm = 'master' }: BaseQuery) => {
  return useQuery(
    window.tanstackApi.get('/realms/{realm_name}/email-templates', {
      path: { realm_name: realm },
    }).queryOptions,
  )
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
      const keys = window.tanstackApi.get('/realms/{realm_name}/email-templates', {
        path: { realm_name: variables.path.realm_name },
      }).queryKey

      await queryClient.invalidateQueries({ queryKey: keys })
      toast.success('Email template created successfully')
    },
  })
}

export const useUpdateEmailTemplate = () => {
  const queryClient = useQueryClient()
  return useMutation({
    ...window.tanstackApi.mutation('put', '/realms/{realm_name}/email-templates/{template_id}').mutationOptions,
    onSuccess: async (_, variables) => {
      const keys = window.tanstackApi.get('/realms/{realm_name}/email-templates', {
        path: { realm_name: variables.path.realm_name },
      }).queryKey

      await queryClient.invalidateQueries({ queryKey: keys })
      toast.success('Email template saved successfully')
    },
  })
}

export const useImportEmailTemplate = () => {
  const queryClient = useQueryClient()
  return useMutation({
    ...window.tanstackApi.mutation('post', '/realms/{realm_name}/email-templates/import')
      .mutationOptions,
    onSuccess: async (_, variables) => {
      const keys = window.tanstackApi.get('/realms/{realm_name}/email-templates', {
        path: { realm_name: variables.path.realm_name },
      }).queryKey

      await queryClient.invalidateQueries({ queryKey: keys })
      toast.success('Email template imported successfully')
    },
    // The server refuses a file that belongs to the other builder or that uses
    // a format version it cannot read; its message says which, so surface it.
    onError: (error) => {
      toast.error('Failed to import', { description: error.message })
    },
  })
}

export const useDeleteEmailTemplate = () => {
  const queryClient = useQueryClient()
  return useMutation({
    ...window.tanstackApi.mutation('delete', '/realms/{realm_name}/email-templates/{template_id}').mutationOptions,
    onSuccess: async (_, variables) => {
      const keys = window.tanstackApi.get('/realms/{realm_name}/email-templates', {
        path: { realm_name: variables.path.realm_name },
      }).queryKey

      await queryClient.invalidateQueries({ queryKey: keys })
      toast.success('Email template deleted successfully')
    },
  })
}
