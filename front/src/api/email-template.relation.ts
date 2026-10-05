import { useParams } from 'react-router'
import type { RelationOption, RelationSource } from '@/components/kit'
import { RouterParams } from '@/routes/router'
import type { Schemas } from './api.client'
import {
  EMAIL_TEMPLATE_SEARCH_LIMIT,
  useGetEmailTemplate,
  useGetEmailTemplates,
} from './email-template.api'

const toOption = (template: Schemas.EmailTemplate): RelationOption => ({
  id: template.id,
  label: template.name,
})

function useRealm() {
  const { realm_name } = useParams<RouterParams>()
  return realm_name ?? 'master'
}

function useSelectedTemplate(id: string | undefined) {
  const realm = useRealm()
  const { data } = useGetEmailTemplate({ realm, templateId: id ?? '' })
  return data?.data ? toOption(data.data) : undefined
}

function sourceFor(emailType: Schemas.EmailType): RelationSource {
  function useTemplateOptions(search: string) {
    const realm = useRealm()
    const { data, isLoading } = useGetEmailTemplates({
      realm,
      query: {
        email_type: emailType,
        name: search.trim() || undefined,
        order_by: 'name',
        order: 'asc',
        limit: EMAIL_TEMPLATE_SEARCH_LIMIT,
      },
    })
    return { options: (data?.data ?? []).map(toOption), loading: isLoading }
  }

  return { useOptions: useTemplateOptions, useSelected: useSelectedTemplate }
}

export const emailTemplateRelationSources: Record<Schemas.EmailType, RelationSource> = {
  reset_password: sourceFor('reset_password'),
  magic_link: sourceFor('magic_link'),
  email_verification: sourceFor('email_verification'),
}
