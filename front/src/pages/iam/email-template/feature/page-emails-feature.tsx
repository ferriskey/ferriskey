import { useMemo } from 'react'
import { useNavigate, useParams } from 'react-router'
import { useTranslation } from 'react-i18next'
import { toast } from 'sonner'
import { useImportEmailTemplate } from '@/api/email-template.api'
import { readExportFile } from '@/api/builder-export'
import { useRouteTabs, type TabItem } from '@/components/kit'
import { RouterParams } from '@/routes/router'
import PageEmails from '../ui/page-emails'
import TemplatesTabFeature from './templates-tab-feature'
import SmtpTabFeature from './smtp-tab-feature'
import { useEmailTemplatesBase } from '@/hooks/use-section-base'
import { apiErrorMessage } from '@/lib/api-error'
import { EMAIL_TEMPLATE_NAMESPACE } from '../email-types'

const TEMPLATES_TAB = 'templates'
const SMTP_TAB = 'smtp'

export default function PageEmailsFeature() {
  const { t } = useTranslation(EMAIL_TEMPLATE_NAMESPACE)
  const emailTemplatesBase = useEmailTemplatesBase()
  const { realm_name } = useParams<RouterParams>()
  const navigate = useNavigate()
  const realm = realm_name ?? 'master'
  const listUrl = emailTemplatesBase

  const tabList = useMemo<TabItem[]>(
    () => [
      { key: TEMPLATES_TAB, label: t('page.tabs.templates') },
      { key: SMTP_TAB, label: t('page.tabs.smtp') },
    ],
    [t]
  )

  const { value: tab, tabs } = useRouteTabs(listUrl, tabList)
  const { mutate: importTemplate } = useImportEmailTemplate()

  const handleImport = (file: File) => {
    readExportFile(file)
      .then((envelope) => {
        importTemplate({ path: { realm_name: realm }, body: envelope as never })
      })
      .catch((error: Error) => toast.error(apiErrorMessage(error)))
  }

  return (
    <PageEmails
      tab={tab}
      tabs={tabs}
      onCreate={() => navigate(`${listUrl}/create/builder`)}
      onImport={handleImport}
    >
      {tab === TEMPLATES_TAB ? <TemplatesTabFeature /> : <SmtpTabFeature />}
    </PageEmails>
  )
}
