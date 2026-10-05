import { useMemo } from 'react'
import { useNavigate, useParams } from 'react-router'
import { useTranslation } from 'react-i18next'
import {
  EMAIL_TEMPLATE_FILTER_KEYS,
  useDeleteEmailTemplate,
  useEmailTemplateCount,
  useGetEmailTemplates,
  type EmailTemplatesQuery,
} from '@/api/email-template.api'
import { usePagedListing } from '@/components/kit'
import { useGetRealm, useUpdateRealmSettings } from '@/api/realm.api'
import { downloadEmailTemplateExport } from '@/api/builder-export'
import { toast } from 'sonner'
import { RouterParams } from '@/routes/router'
import TemplatesTab from '../ui/templates-tab'
import { EMAIL_TEMPLATE_NAMESPACE, type EmailTypeSpec } from '../email-types'
import { useEmailTemplatesBase } from '@/hooks/use-section-base'

export default function TemplatesTabFeature() {
  const { t } = useTranslation(EMAIL_TEMPLATE_NAMESPACE)
  const emailTemplatesBase = useEmailTemplatesBase()
  const { realm_name } = useParams<RouterParams>()
  const navigate = useNavigate()
  const realm = realm_name ?? 'master'
  const listUrl = emailTemplatesBase

  const listing = usePagedListing(EMAIL_TEMPLATE_FILTER_KEYS)
  const { data: templatesResponse, isLoading } = useGetEmailTemplates({
    realm,
    query: listing.apiQuery as EmailTemplatesQuery,
    keepPrevious: true,
  })
  const total = useEmailTemplateCount({ realm })
  const resetPassword = useEmailTemplateCount({ realm, filter: { email_type: 'reset_password' } })
  const magicLink = useEmailTemplateCount({ realm, filter: { email_type: 'magic_link' } })
  const emailVerification = useEmailTemplateCount({
    realm,
    filter: { email_type: 'email_verification' },
  })
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
      listing={listing}
      pagination={templatesResponse?.metadata}
      counts={{
        total: total.count,
        byType: {
          reset_password: resetPassword.count,
          magic_link: magicLink.count,
          email_verification: emailVerification.count,
        },
        loadingByType: {
          reset_password: resetPassword.isLoading,
          magic_link: magicLink.isLoading,
          email_verification: emailVerification.isLoading,
        },
      }}
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
          toast.error(t('list.toast.export_failed'))
        )
      }
    />
  )
}
