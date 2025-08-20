import { Realm } from '@/api/core.interface'
import BadgeColor from '@/components/ui/badge-color'
import { BadgeColorScheme } from '@/components/ui/badge-color.enum'
import { Heading } from '@/components/ui/heading'
import { Tabs, TabsList, TabsTrigger } from '@/components/ui/tabs'
import WorkInProgress from '@/components/work-in-progress'
import { useFeature } from '@/hooks/use-feature'
import { Feature } from '@/lib/features'
import { REALM_SETTINGS_URL } from '@/routes/router'
import { Outlet, useNavigate } from 'react-router'

interface PageRealmSettingsProps {
  realm: Realm
  tab?: string
  setTab?: (value: string) => void
}

export default function PageRealmSettings({ realm, tab, setTab }: PageRealmSettingsProps) {
  const enabled = useFeature(Feature.REALM_SETTINGS)
  const navigate = useNavigate()
  return (
    <div className='flex flex-col gap-4 p-8'>
      <div className='flex flex-col gap-2 border-b pb-4'>
        <div className='flex flex-col gap-2'>
            <Heading>{realm.name}</Heading>
            <BadgeColor className='w-fit' color={BadgeColorScheme.GRAY}>{realm.id}</BadgeColor>
          <p>Realm settings are settings that control the options for users, applications, roles, and groups in the current realm.</p>
        </div>
        <div>
          <Tabs defaultValue={tab} value={tab} onValueChange={(value) => {
            navigate(`${REALM_SETTINGS_URL(realm.name)}/${value}`)
            if(setTab) {
              setTab(value || 'general')
            }
          }}>
            <TabsList className='flex items-center gap-4'>
              <TabsTrigger value={'general'}>General</TabsTrigger>
              <TabsTrigger className={'cursor-not-allowed'} disabled={true} value={'login'}>Login</TabsTrigger>
              <TabsTrigger className={'cursor-not-allowed'} disabled={true} value={'security'}>Security</TabsTrigger>
            </TabsList>
          </Tabs>
        </div>
      </div>
      {
        enabled ? <Outlet /> : <WorkInProgress/>
      }
    </div>
  )
}
