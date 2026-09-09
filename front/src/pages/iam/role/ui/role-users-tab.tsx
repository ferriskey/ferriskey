import { Section } from '@/components/kit'
import WorkInProgress from '@/components/work-in-progress'

export default function RoleUsersTab() {
  return (
    <Section
      title='Accounts holding this role'
      description='Listing the holders of a role is not exposed by the API yet.'
      contained={false}
    >
      <WorkInProgress />
    </Section>
  )
}
