import { useTranslation } from 'react-i18next'

const HINT_KEYS = {
  default: 'scope_form.type.hint.default',
  optional: 'scope_form.type.hint.optional',
} as const

export default function ScopeTypeHint({ scopeType }: { scopeType: 'optional' | 'default' }) {
  const { t } = useTranslation('client-scope')

  return (
    <p className='mt-1.5 text-xs text-neutral-500 dark:text-neutral-400'>
      {t(HINT_KEYS[scopeType])}
    </p>
  )
}
