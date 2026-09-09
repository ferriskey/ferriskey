import { useCallback, useMemo, useState } from 'react'
import { useNavigate, useParams } from 'react-router'
import {
  useCreateEmailTemplate,
  useGetEmailTemplate,
  useGetTemplateVariables,
  useUpdateEmailTemplate,
} from '@/api/email-template.api'
import type { BuilderNode } from '@/lib/builder-core'
import { createMjmlAdapter, type EmailTemplatePreset } from '@/lib/builder-mjml'
import { EMAIL_TYPES } from '../email-types'
import PageEmailTemplateBuilder from '../ui/page-email-template-builder'
import { useEmailTemplatesBase } from '@/next/shared/use-section-base'

export default function PageEmailTemplateBuilderFeature() {
  const emailTemplatesBase = useEmailTemplatesBase()
  const { realm_name, template_id } = useParams<{ realm_name: string; template_id: string }>()
  const navigate = useNavigate()
  const realm = realm_name ?? 'master'
  const isNew = template_id === undefined || template_id === 'create'
  const listUrl = `${emailTemplatesBase}/templates`

  const { data: templateResponse, isLoading } = useGetEmailTemplate({
    realm,
    templateId: isNew ? 'new' : (template_id ?? ''),
  })

  if (!isNew && isLoading) {
    return (
      <div className='grid h-full place-items-center p-12 text-sm text-neutral-500 dark:text-neutral-400'>
        Loading template…
      </div>
    )
  }

  return (
    <BuilderFeatureInner
      key={templateResponse?.data?.id ?? 'new'}
      realm={realm}
      templateId={isNew ? '' : (template_id ?? '')}
      isNew={isNew}
      initialName={templateResponse?.data?.name ?? ''}
      initialEmailType={templateResponse?.data?.email_type ?? 'reset_password'}
      initialTree={
        templateResponse?.data?.structure
          ? ((templateResponse.data.structure as { children?: BuilderNode[] }).children ?? [])
          : []
      }
      navigate={navigate}
      listUrl={listUrl}
    />
  )
}

function BuilderFeatureInner({
  realm,
  templateId,
  isNew,
  initialName,
  initialEmailType,
  initialTree,
  navigate,
  listUrl,
}: {
  realm: string
  templateId: string
  isNew: boolean
  initialName: string
  initialEmailType: string
  initialTree: BuilderNode[]
  navigate: ReturnType<typeof useNavigate>
  listUrl: string
}) {
  const [name, setName] = useState(initialName)
  const [emailType, setEmailType] = useState(initialEmailType)
  const [tree, setTree] = useState<BuilderNode[]>(initialTree)

  const { data: variablesResponse } = useGetTemplateVariables(emailType)

  const adapter = useMemo(
    () => createMjmlAdapter({ variables: variablesResponse?.data }),
    [variablesResponse]
  )

  const { mutate: createTemplate, isPending: isCreating } = useCreateEmailTemplate()
  const { mutate: updateTemplate, isPending: isUpdating } = useUpdateEmailTemplate()

  const handleTreeChange = useCallback((next: BuilderNode[]) => setTree(next), [])

  const handleSave = () => {
    const structure = { children: tree }

    if (isNew) {
      createTemplate(
        {
          path: { realm_name: realm },
          body: { name, email_type: emailType, structure },
        },
        { onSuccess: () => navigate(listUrl) }
      )
      return
    }

    updateTemplate({
      path: { realm_name: realm, template_id: templateId },
      body: { name, structure },
    })
  }

  const handleApplyPreset = (preset: EmailTemplatePreset) => {
    setEmailType(preset.emailType)
    setName(preset.name)
  }

  return (
    <PageEmailTemplateBuilder
      adapter={adapter}
      tree={tree}
      onTreeChange={handleTreeChange}
      name={name}
      onNameChange={setName}
      emailType={emailType}
      onEmailTypeChange={setEmailType}
      emailTypes={EMAIL_TYPES.map((spec) => ({ label: spec.label, value: spec.key }))}
      isNew={isNew}
      isSaving={isCreating || isUpdating}
      onSave={handleSave}
      onBack={() => navigate(listUrl)}
      onApplyPreset={handleApplyPreset}
    />
  )
}
