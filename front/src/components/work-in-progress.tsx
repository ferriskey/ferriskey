import { useTranslation } from 'react-i18next'

export default function WorkInProgress() {
  const { t } = useTranslation()

  return <p className='text-muted-foreground'>{t('work_in_progress')}</p>
}
