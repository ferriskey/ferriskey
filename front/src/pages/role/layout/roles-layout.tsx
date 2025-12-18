import { useState } from 'react'
import { Tabs, TabsList, TabsTrigger } from '@/components/ui/tabs'
import PageHeader from '@/components/ui/page-header'
import PageContainer from '@/components/ui/page-container'
import { Shield } from 'lucide-react'

export default function RolesLayout() {
  const [tab, setTab] = useState('list')

  return (
    <PageContainer>
      <PageHeader
        icon={Shield}
        title='Roles'
        description='Manage and configure roles and permissions for your realm'
      >
        <Tabs defaultValue={tab} onValueChange={setTab}>
          <TabsList>
            <TabsTrigger value={'list'}>Roles list</TabsTrigger>
          </TabsList>
        </Tabs>
      </PageHeader>
    </PageContainer>
  )
}
