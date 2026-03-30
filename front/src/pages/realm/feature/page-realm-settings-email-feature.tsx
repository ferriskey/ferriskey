import { useGetEmailTemplates, useDeleteEmailTemplate } from '@/api/email-template.api'
import { useGetRealm, useUpdateRealmSettings } from '@/api/realm.api'
import { RouterParams } from '@/routes/router'
import { useNavigate, useParams } from 'react-router'
import { EMAIL_TEMPLATE_BUILDER_URL } from '@/routes/sub-router/email-template.router'
import PageRealmSettingsEmail from '../ui/page-realm-settings-email'

export default function PageRealmSettingsEmailFeature() {
  const { realm_name } = useParams<RouterParams>()
  const navigate = useNavigate()
  const realm = realm_name ?? 'master'
  const { data: templatesData, isLoading: templatesLoading } = useGetEmailTemplates({ realm })
  const { mutate: deleteTemplate } = useDeleteEmailTemplate()
  const { data: realmData } = useGetRealm({ realm })
  const { mutate: updateSettings } = useUpdateRealmSettings()

  const handleEditTemplate = (templateId: string) => {
    navigate(EMAIL_TEMPLATE_BUILDER_URL(realm_name, templateId))
  }

  const handleCreateTemplate = () => {
    navigate(EMAIL_TEMPLATE_BUILDER_URL(realm_name, 'new'))
  }

  const handleDeleteTemplate = (templateId: string) => {
    deleteTemplate({ path: { realm_name: realm, template_id: templateId } })
  }

  const handleAssignTemplate = (field: string, templateId: string | null) => {
    if (!realm_name) return

    updateSettings({
      path: { name: realm_name },
      body: {
        [field]: templateId,
      },
    })
  }

  const realmSettings = realmData?.settings

  return (
    <PageRealmSettingsEmail
      templates={templatesData?.data ?? []}
      templatesLoading={templatesLoading}
      onEditTemplate={handleEditTemplate}
      onCreateTemplate={handleCreateTemplate}
      onDeleteTemplate={handleDeleteTemplate}
      realmSettings={realmSettings}
      onAssignTemplate={handleAssignTemplate}
    />
  )
}
