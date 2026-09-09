import { useState } from 'react'
import { useNavigate, useParams } from 'react-router'
import { toast } from 'sonner'
import { useCreateOrganization } from '@/api/organization.api'
import { RouterParams } from '@/routes/router'
import { createOrganizationSchema } from '@/pages/organization/schemas/create-organization.schema'
import PageCreateOrganization, {
  type CreateOrganizationDraft,
} from '@/next/pages/iam/organization/ui/page-create-organization'
import { CONSOLE_ORGANIZATION_URL, CONSOLE_ORGANIZATIONS_URL } from '../urls'

const EMPTY_DRAFT: CreateOrganizationDraft = {
  name: '',
  alias: '',
  enabled: true,
  domain: '',
  redirectUrl: '',
  description: '',
}

export default function PageCreateOrganizationFeature() {
  const { realm_name } = useParams<RouterParams>()
  const navigate = useNavigate()
  const realm = realm_name ?? 'master'

  const { mutate: createOrganization } = useCreateOrganization()
  const [draft, setDraft] = useState<CreateOrganizationDraft>(EMPTY_DRAFT)

  const parsed = createOrganizationSchema.safeParse({
    name: draft.name,
    alias: draft.alias,
    domain: draft.domain,
    redirectUrl: draft.redirectUrl,
    description: draft.description,
    enabled: draft.enabled,
  })

  const errors = parsed.success
    ? {}
    : {
        name: parsed.error.issues.find((issue) => issue.path[0] === 'name')?.message,
        alias: parsed.error.issues.find((issue) => issue.path[0] === 'alias')?.message,
      }

  const handleSubmit = () => {
    if (!parsed.success) return

    createOrganization(
      {
        path: { realm_name: realm },
        body: {
          name: draft.name,
          alias: draft.alias,
          domain: draft.domain || null,
          redirect_url: draft.redirectUrl || null,
          description: draft.description || null,
          enabled: draft.enabled,
        },
      },
      {
        onSuccess: (payload) =>
          navigate(`${CONSOLE_ORGANIZATION_URL(realm, payload.id)}/settings`),
        onError: () => toast.error('Failed to create organization'),
      }
    )
  }

  return (
    <PageCreateOrganization
      draft={draft}
      errors={errors}
      canSubmit={parsed.success}
      onChange={(patch) => setDraft((d) => ({ ...d, ...patch }))}
      onBack={() => navigate(CONSOLE_ORGANIZATIONS_URL(realm))}
      onSubmit={handleSubmit}
    />
  )
}
