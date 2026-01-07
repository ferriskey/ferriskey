import { useState } from 'react'
import { Tabs, TabsList, TabsTrigger } from '@/components/ui/tabs'
import PageHeader from '@/components/ui/page-header'
import PageContainer from '@/components/ui/page-container'
import { Database } from 'lucide-react'

export default function UserFederationLayout() {
  const [tab, setTab] = useState('list')

  return (
    <PageContainer>
      <PageHeader
        icon={Database}
        title='User Federation'
        description='Manage and configure external user storage providers'
      >
        <Tabs defaultValue={tab} onValueChange={setTab}>
          <TabsList>
            <TabsTrigger value={'list'}>Providers list</TabsTrigger>
          </TabsList>
        </Tabs>
      </PageHeader>
    </PageContainer>
  )
}
