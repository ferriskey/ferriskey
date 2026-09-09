import { useMemo } from 'react'
import { useNavigate, useParams } from 'react-router'
import { useDeleteEmailTemplate, useGetEmailTemplates } from '@/api/email-template.api'
import { useGetRealm, useUpdateRealmSettings } from '@/api/realm.api'
import { downloadEmailTemplateExport } from '@/api/builder-export'
import { toast } from 'sonner'
import { RouterParams } from '@/routes/router'
import TemplatesTab from '../ui/templates-tab'
import type { EmailTypeSpec } from '../email-types'
import { useEmailTemplatesBase } from '@/next/shared/use-section-base'

export default function TemplatesTabFeature() {
  const emailTemplatesBase = useEmailTemplatesBase()
  const { realm_name } = useParams<RouterParams>()
  const navigate = useNavigate()
  const realm = realm_name ?? 'master'
  const listUrl = emailTemplatesBase

  const { data: templatesResponse, isLoading } = useGetEmailTemplates({ realm })
  const { data: realmResponse } = useGetRealm({ realm })
  const { mutate: deleteTemplate } = useDeleteEmailTemplate()
  const { mutate: updateSettings } = useUpdateRealmSettings()

  const templates = useMemo(() => templatesResponse?.data ?? [], [templatesResponse])
  const settings = realmResponse?.settings

  const assignments = useMemo(
    () => ({
      reset_password_template_id: settings?.reset_password_template_id ?? null,
      magic_link_template_id: settings?.magic_link_template_id ?? null,
      email_verification_template_id: settings?.email_verification_template_id ?? null,
    }),
    [settings]
  )

  return (
    <TemplatesTab
      templates={templates}
      isLoading={isLoading}
      assignments={assignments}
      templateHref={(id: string) => `${listUrl}/${id}`}
      onAssign={(field: EmailTypeSpec['assignmentField'], templateId: string | null) =>
        updateSettings({ path: { name: realm }, body: { [field]: templateId } })
      }
      onCreate={() => navigate(`${listUrl}/create/builder`)}
      onEdit={(id: string) => navigate(`${listUrl}/${id}/builder`)}
      onDelete={(id: string) => deleteTemplate({ path: { realm_name: realm, template_id: id } })}
      onExport={(id: string, format: 'json' | 'mjml') =>
        downloadEmailTemplateExport(realm, id, format).catch(() =>
          toast.error('Could not export this template')
        )
      }
    />
  )
}
