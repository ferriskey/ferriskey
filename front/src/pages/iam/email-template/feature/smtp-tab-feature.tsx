import { useState } from 'react'
import { useParams } from 'react-router'
import { useDeleteSmtpConfig, useGetSmtpConfig, useUpsertSmtpConfig } from '@/api/smtp.api'
import { RouterParams } from '@/routes/router'
import {
  EMPTY_SMTP_DRAFT,
  smtpErrors,
  type SmtpConfigDraft,
} from '../smtp-form'
import SmtpTab from '../ui/smtp-tab'

interface DraftState {
  key: string
  values: SmtpConfigDraft
}

export default function SmtpTabFeature() {
  const { realm_name } = useParams<RouterParams>()
  const realm = realm_name ?? 'master'

  const { data: config, isError, isLoading } = useGetSmtpConfig({ realm })
  const { mutate: upsert } = useUpsertSmtpConfig()
  const { mutate: remove } = useDeleteSmtpConfig()

  const hasConfig = Boolean(config) && !isError

  const pristine: DraftState = hasConfig && config
    ? {
        key: config.id,
        values: {
          host: config.host,
          port: config.port,
          username: config.username,
          password: '',
          from_email: config.from_email,
          from_name: config.from_name,
          encryption: config.encryption,
        },
      }
    : { key: 'unconfigured', values: EMPTY_SMTP_DRAFT }

  const [draft, setDraft] = useState<DraftState>(pristine)

  if (draft.key !== pristine.key) setDraft(pristine)

  const values = draft.key === pristine.key ? draft.values : pristine.values
  const errors = smtpErrors(values)
  const dirty = JSON.stringify(values) !== JSON.stringify(pristine.values)

  const handleChange = <K extends keyof SmtpConfigDraft>(field: K, value: SmtpConfigDraft[K]) => {
    setDraft({ key: pristine.key, values: { ...values, [field]: value } })
  }

  const handleSave = () => {
    if (Object.keys(errors).length > 0) return
    upsert({ path: { realm_name: realm }, body: values })
  }

  const handleDelete = () => {
    remove({ path: { realm_name: realm } })
  }

  return (
    <SmtpTab
      draft={values}
      errors={dirty ? errors : {}}
      hasConfig={hasConfig}
      isLoading={isLoading}
      dirty={dirty}
      onChange={handleChange}
      onDiscard={() => setDraft(pristine)}
      onSave={handleSave}
      onDelete={handleDelete}
    />
  )
}
