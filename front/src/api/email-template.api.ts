import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { toast } from 'sonner'

interface EmailTemplate {
  id: string
  realm_id: string
  name: string
  email_type: string
  structure: unknown
  mjml: string
  is_active: boolean
  created_at: string
  updated_at: string
}

interface TemplateVariable {
  name: string
  description: string
}

export const useGetEmailTemplates = (realm: string) => {
  return useQuery({
    queryKey: ['email-templates', realm],
    queryFn: async (): Promise<{ data: EmailTemplate[] }> => {
      const res = await window.axios.get(`/realms/${realm}/email-templates`)
      return res.data
    },
  })
}

export const useGetEmailTemplate = (realm: string, templateId: string) => {
  return useQuery({
    queryKey: ['email-templates', realm, templateId],
    queryFn: async (): Promise<{ data: EmailTemplate }> => {
      const res = await window.axios.get(`/realms/${realm}/email-templates/${templateId}`)
      return res.data
    },
    enabled: !!templateId && templateId !== 'new',
  })
}

export const useGetTemplateVariables = (emailType: string) => {
  return useQuery({
    queryKey: ['email-template-variables', emailType],
    queryFn: async (): Promise<{ data: TemplateVariable[] }> => {
      const res = await window.axios.get(`/email-templates/variables/${emailType}`)
      return res.data
    },
    enabled: !!emailType,
  })
}

export const useCreateEmailTemplate = () => {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: async (params: {
      realm: string
      body: { name: string; email_type: string; structure: unknown }
    }): Promise<{ data: EmailTemplate }> => {
      const res = await window.axios.post(
        `/realms/${params.realm}/email-templates`,
        params.body,
      )
      return res.data
    },
    onSuccess: async () => {
      toast.success('Email template created successfully')
      await queryClient.invalidateQueries({ queryKey: ['email-templates'] })
    },
  })
}

export const useUpdateEmailTemplate = () => {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: async (params: {
      realm: string
      templateId: string
      body: { name: string; structure: unknown }
    }): Promise<{ data: EmailTemplate }> => {
      const res = await window.axios.put(
        `/realms/${params.realm}/email-templates/${params.templateId}`,
        params.body,
      )
      return res.data
    },
    onSuccess: async () => {
      toast.success('Email template saved successfully')
      await queryClient.invalidateQueries({ queryKey: ['email-templates'] })
    },
  })
}

export const useDeleteEmailTemplate = () => {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: async (params: { realm: string; templateId: string }) => {
      await window.axios.delete(`/realms/${params.realm}/email-templates/${params.templateId}`)
    },
    onSuccess: async () => {
      toast.success('Email template deleted successfully')
      await queryClient.invalidateQueries({ queryKey: ['email-templates'] })
    },
  })
}

export const useActivateEmailTemplate = () => {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: async (params: { realm: string; templateId: string }): Promise<{ data: EmailTemplate }> => {
      const res = await window.axios.patch(
        `/realms/${params.realm}/email-templates/${params.templateId}/activate`,
      )
      return res.data
    },
    onSuccess: async () => {
      toast.success('Email template activated')
      await queryClient.invalidateQueries({ queryKey: ['email-templates'] })
    },
  })
}
