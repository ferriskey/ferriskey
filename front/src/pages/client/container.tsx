import { Tabs, TabsList, TabsTrigger } from '@/components/ui/tabs'
import { Users } from 'lucide-react'
import { useState } from 'react'
import PageHeader from '@/components/ui/page-header'
import PageContainer from '@/components/ui/page-container'

export default function Container() {
  const [tab, setTab] = useState('list')

  return (
    <PageContainer>
      <PageHeader
        icon={Users}
        title='Clients'
        description='Manage and configure OAuth clients for your applications'
      >
        <Tabs defaultValue={tab} onValueChange={setTab}>
          <TabsList>
            <TabsTrigger value={'list'}>Clients list</TabsTrigger>
          </TabsList>
        </Tabs>
      </PageHeader>
    </PageContainer>
  )
}
