import { useState } from 'react'
import { useNavigate, useParams } from 'react-router'
import { toast } from 'sonner'
import {
  useDeleteEmailTemplate,
  useGetEmailTemplate,
  useGetTemplateVariables,
  useUpdateEmailTemplate,
} from '@/api/email-template.api'
import { useGetRealm } from '@/api/realm.api'
import { downloadEmailTemplateExport } from '@/api/builder-export'
import { NEXT_EMAIL_TEMPLATES_URL } from '@/next/routes'
import { EMAIL_TYPES } from '../email-types'
import PageEmailTemplateDetail from '../ui/page-email-template-detail'
import { useCrumbLabel } from '@/next/shell/crumb-store'

interface Draft {
  key: string
  name: string
}

const EMPTY_DRAFT: Draft = { key: '', name: '' }

export default function PageEmailTemplateDetailFeature() {
  const { realm_name, template_id } = useParams<{ realm_name: string; template_id: string }>()
  const navigate = useNavigate()
  const realm = realm_name ?? 'master'
  const templateId = template_id ?? ''
  const listUrl = `${NEXT_EMAIL_TEMPLATES_URL(realm)}/templates`

  const { data: templateResponse, isLoading } = useGetEmailTemplate({ realm, templateId })
  const { data: realmResponse } = useGetRealm({ realm })
  const { mutate: updateTemplate } = useUpdateEmailTemplate()
  const { mutateAsync: deleteTemplate } = useDeleteEmailTemplate()

  const template = templateResponse?.data
  const { data: variablesResponse } = useGetTemplateVariables(template?.email_type ?? '')

  const [draft, setDraft] = useState<Draft>(EMPTY_DRAFT)

  const pristine: Draft = template ? { key: template.id, name: template.name } : EMPTY_DRAFT
  if (template && draft.key !== template.id) setDraft(pristine)
  const name = draft.key === pristine.key ? draft.name : pristine.name

  const settings = realmResponse?.settings
  const assignedTo = EMAIL_TYPES.filter(
    (spec) => template && settings?.[spec.assignmentField] === template.id
  )

  const dirty = Boolean(template && name !== template.name)
  const nameError = name.trim().length === 0 ? 'Name is required' : undefined

  const save = () => {
    if (!template || nameError) return
    updateTemplate({
      path: { realm_name: realm, template_id: template.id },
      body: { name, structure: template.structure },
    })
  }

  const handleDelete = async () => {
    if (!template) return
    try {
      await deleteTemplate({ path: { realm_name: realm, template_id: template.id } })
      navigate(listUrl)
    } catch {
      return
    }
  }


  useCrumbLabel(template_id, template?.name)

  return (
    <PageEmailTemplateDetail
      template={template}
      isLoading={isLoading}
      variables={variablesResponse?.data ?? []}
      assignedTo={assignedTo}
      name={name}
      nameError={dirty ? nameError : undefined}
      dirty={dirty}
      onNameChange={(value: string) => setDraft({ key: pristine.key, name: value })}
      onBack={() => navigate(listUrl)}
      onOpenBuilder={() =>
        navigate(`${NEXT_EMAIL_TEMPLATES_URL(realm)}/${templateId}/builder`)
      }
      onExport={(format: 'json' | 'mjml') =>
        downloadEmailTemplateExport(realm, templateId, format).catch(() =>
          toast.error('Could not export this template')
        )
      }
      onDiscard={() => setDraft(pristine)}
      onSave={save}
      onDelete={handleDelete}
    />
  )
}
