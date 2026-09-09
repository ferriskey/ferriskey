import { useNavigate, useParams } from 'react-router'
import { toast } from 'sonner'
import { useImportEmailTemplate } from '@/api/email-template.api'
import { readExportFile } from '@/api/builder-export'
import { useRouteTabs } from '@/components/kit'
import { RouterParams } from '@/routes/router'
import PageEmails from '../ui/page-emails'
import TemplatesTabFeature from './templates-tab-feature'
import SmtpTabFeature from './smtp-tab-feature'
import { useEmailTemplatesBase } from '@/next/shared/use-section-base'

const EMAIL_TABS = [
  { key: 'templates', label: 'Templates' },
  { key: 'smtp', label: 'SMTP' },
] as const

export default function PageEmailsFeature() {
  const emailTemplatesBase = useEmailTemplatesBase()
  const { realm_name } = useParams<RouterParams>()
  const navigate = useNavigate()
  const realm = realm_name ?? 'master'
  const listUrl = emailTemplatesBase

  const { value: tab, tabs } = useRouteTabs(listUrl, EMAIL_TABS)
  const { mutate: importTemplate } = useImportEmailTemplate()

  const handleImport = (file: File) => {
    readExportFile(file)
      .then((envelope) => {
        importTemplate({ path: { realm_name: realm }, body: envelope as never })
      })
      .catch((error: Error) => toast.error(error.message))
  }

  return (
    <PageEmails
      tab={tab}
      tabs={tabs}
      onCreate={() => navigate(`${listUrl}/create/builder`)}
      onImport={handleImport}
    >
      {tab === 'templates' ? <TemplatesTabFeature /> : <SmtpTabFeature />}
    </PageEmails>
  )
}
