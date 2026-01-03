import { useState } from 'react'
import { Tabs, TabsList, TabsTrigger } from '@/components/ui/tabs'
import PageHeader from '@/components/ui/page-header'
import PageContainer from '@/components/ui/page-container'
import { ScanFace } from 'lucide-react'

export default function ProvidersLayout() {
  const [tab, setTab] = useState('list')

  return (
    <PageContainer>
      <PageHeader
        icon={ScanFace}
        title='Identity Providers'
        description='Manage external authentication sources and SSO integrations'
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
