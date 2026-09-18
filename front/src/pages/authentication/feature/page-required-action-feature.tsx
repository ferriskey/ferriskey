import { useTranslation } from 'react-i18next'
import { useSearchParams } from 'react-router'
import PageRequiredAction from '../ui/page-required-action'
import { AUTH_NAMESPACE } from '../constants'

export default function PageRequiredActionFeature() {
  const [searchParams] = useSearchParams()
  const { t } = useTranslation(AUTH_NAMESPACE)
  const execution = searchParams.get('execution')

  if (!execution) {
    return <div>{t('required_action.loading')}</div>
  }

  return (
    <div>
      <PageRequiredAction execution={execution} />
    </div>
  )
}
