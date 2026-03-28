import { useNavigate, useParams } from 'react-router-dom'
import type { RouterParams } from '@/routes/router'
import {
  useGetEmailTemplates,
  useDeleteEmailTemplate,
  useActivateEmailTemplate,
} from '@/api/email-template.api'
import { EMAIL_TEMPLATE_BUILDER_URL } from '@/routes/sub-router/email-template.router'
import PageEmailTemplateList from '../ui/page-email-template-list'

export default function PageEmailTemplateListFeature() {
  const { realm_name } = useParams<RouterParams>()
  const navigate = useNavigate()
  const realm = realm_name ?? 'master'

  const { data, isLoading } = useGetEmailTemplates(realm)
  const { mutate: deleteTemplate } = useDeleteEmailTemplate()
  const { mutate: activateTemplate } = useActivateEmailTemplate()

  const handleEdit = (templateId: string) => {
    navigate(EMAIL_TEMPLATE_BUILDER_URL(realm_name, templateId))
  }

  const handleDelete = (templateId: string) => {
    deleteTemplate({ realm, templateId })
  }

  const handleActivate = (templateId: string) => {
    activateTemplate({ realm, templateId })
  }

  const handleCreate = () => {
    navigate(EMAIL_TEMPLATE_BUILDER_URL(realm_name, 'new'))
  }

  return (
    <PageEmailTemplateList
      templates={data?.data ?? []}
      isLoading={isLoading}
      onEdit={handleEdit}
      onDelete={handleDelete}
      onActivate={handleActivate}
      onCreate={handleCreate}
    />
  )
}
