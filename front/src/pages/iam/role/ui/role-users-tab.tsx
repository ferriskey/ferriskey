import { useTranslation } from 'react-i18next'
import { Section } from '@/components/kit'
import WorkInProgress from '@/components/work-in-progress'

export default function RoleUsersTab() {
  const { t } = useTranslation('role')

  return (
    <Section
      title={t('detail.users.title')}
      description={t('detail.users.description')}
      contained={false}
    >
      <WorkInProgress />
    </Section>
  )
}
