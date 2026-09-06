import { useNavigate, useParams } from 'react-router-dom'
import type { RouterParams } from '@/routes/router'
import {
  useGetEmailTemplates,
  useDeleteEmailTemplate,
  useImportEmailTemplate,
} from '@/api/email-template.api'
import { downloadEmailTemplateExport, readExportFile } from '@/api/builder-export'
import { toast } from 'sonner'
import { EMAIL_TEMPLATE_BUILDER_URL } from '@/routes/sub-router/email-template.router'
import PageEmailTemplateList from '../ui/page-email-template-list'

export default function PageEmailTemplateListFeature() {
  const { realm_name } = useParams<RouterParams>()
  const navigate = useNavigate()
  const realm = realm_name ?? 'master'

  const { data, isLoading } = useGetEmailTemplates({ realm })
  const { mutate: deleteTemplate } = useDeleteEmailTemplate()
  const { mutate: importTemplate } = useImportEmailTemplate()

  const handleEdit = (templateId: string) => {
    navigate(EMAIL_TEMPLATE_BUILDER_URL(realm_name, templateId))
  }

  const handleDelete = (templateId: string) => {
    deleteTemplate({
      path: { realm_name: realm, template_id: templateId },
    })
  }

  const handleExport = (templateId: string, format: 'json' | 'mjml') => {
    downloadEmailTemplateExport(realm, templateId, format).catch(() =>
      toast.error('Could not export this template'),
    )
  }

  /**
   * The exported file is sent back as-is: the server reads its envelope, and
   * refuses a file belonging to the other builder or written in a format it
   * cannot read.
   */
  const handleImport = (file: File) => {
    readExportFile(file)
      .then((envelope) => {
        importTemplate({
          path: { realm_name: realm },
          body: envelope as never,
        })
      })
      .catch((error: Error) => toast.error(error.message))
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
      onExport={handleExport}
      onImport={handleImport}
      onCreate={handleCreate}
    />
  )
}
