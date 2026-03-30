import {
  useCreateEmailTemplate,
  useGetEmailTemplate,
  useGetTemplateVariables,
  useUpdateEmailTemplate,
} from '@/api/email-template.api'
import type { BuilderNode } from '@/lib/builder-core'
import { type EmailTemplatePreset, createMjmlAdapter } from '@/lib/builder-mjml'
import type { EmailTemplateRouterParams } from '@/routes/sub-router/email-template.router'
import { EMAIL_TEMPLATES_URL } from '@/routes/sub-router/email-template.router'
import { useCallback, useMemo, useState } from 'react'
import { useNavigate, useParams } from 'react-router-dom'
import PageEmailTemplateBuilder from '../ui/page-email-template-builder'

const EMAIL_TYPES = [
  { label: 'Reset Password', value: 'reset_password' },
  { label: 'Magic Link', value: 'magic_link' },
  { label: 'Email Verification', value: 'email_verification' },
]

export default function PageEmailTemplateBuilderFeature() {
  const { realm_name, template_id } = useParams<EmailTemplateRouterParams>()
  const navigate = useNavigate()
  const realm = realm_name ?? 'master'
  const isNew = template_id === 'new'

  const { data: templateData, isLoading } = useGetEmailTemplate({
    realm,
    templateId: template_id ?? '',
  })

  if (!isNew && isLoading) {
    return (
      <div className='flex items-center justify-center p-12 text-sm text-muted-foreground'>
        Loading template...
      </div>
    )
  }

  return (
    <BuilderFeatureInner
      key={templateData?.data?.id ?? 'new'}
      realmName={realm_name}
      realm={realm}
      templateId={template_id ?? ''}
      isNew={isNew}
      initialName={templateData?.data?.name ?? ''}
      initialEmailType={templateData?.data?.email_type ?? 'reset_password'}
      initialTree={
        templateData?.data?.structure
          ? ((templateData.data.structure as { children?: BuilderNode[] }).children ?? [])
          : []
      }
      navigate={navigate}
    />
  )
}

function BuilderFeatureInner({
  realmName,
  realm,
  templateId,
  isNew,
  initialName,
  initialEmailType,
  initialTree,
  navigate,
}: {
  realmName: string | undefined
  realm: string
  templateId: string
  isNew: boolean
  initialName: string
  initialEmailType: string
  initialTree: BuilderNode[]
  navigate: ReturnType<typeof useNavigate>
}) {
  const [name, setName] = useState(initialName)
  const [emailType, setEmailType] = useState(initialEmailType)
  const [tree, setTree] = useState<BuilderNode[]>(initialTree)

  const { data: variablesData } = useGetTemplateVariables(emailType)

  const adapter = useMemo(
    () =>
      createMjmlAdapter({
        variables: variablesData?.data,
      }),
    [variablesData],
  )

  const { mutate: createTemplate, isPending: isCreating } = useCreateEmailTemplate()
  const { mutate: updateTemplate, isPending: isUpdating } = useUpdateEmailTemplate()

  const handleTreeChange = useCallback((newTree: BuilderNode[]) => {
    setTree(newTree)
  }, [])

  const handleSave = () => {
    const structure = { children: tree }

    if (isNew) {
      createTemplate(
        {
          path: { realm_name: realm },
          body: { name, email_type: emailType, structure },
        },
        {
          onSuccess: () => {
            navigate(EMAIL_TEMPLATES_URL(realmName) + '/overview')
          },
        },
      )
    } else {
      updateTemplate({
        path: { realm_name: realm, template_id: templateId },
        body: { name, structure },
      })
    }
  }

  const handleApplyPreset = (preset: EmailTemplatePreset) => {
    setEmailType(preset.emailType)
    setName(preset.name)
  }

  const handleBack = () => {
    navigate(EMAIL_TEMPLATES_URL(realmName) + '/overview')
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
      emailTypes={EMAIL_TYPES}
      isNew={isNew}
      isSaving={isCreating || isUpdating}
      onSave={handleSave}
      onBack={handleBack}
      onApplyPreset={handleApplyPreset}
    />
  )
}
