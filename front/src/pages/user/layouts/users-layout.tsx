import { useState } from 'react'
import { Outlet, useParams } from 'react-router'
import { Heading } from '../../../components/ui/heading'
import { Tabs, TabsList, TabsTrigger } from '../../../components/ui/tabs'
import { RouterParams } from '../../../routes/router'

export default function UsersLayout() {
  const { realm_name = 'master' } = useParams<RouterParams>()
  const [tab, setTab] = useState('list')

  return (
    <div className='flex flex-col gap-4 p-8'>
      <div className='flex flex-col gap-2 border-b pb-4'>
        <div className='flex flex-col gap-2'>
          <Heading>Users</Heading>
          <p>Manage users in {realm_name}</p>
        </div>
        <div className='flex justify-between items-center'>
          <Tabs defaultValue={tab} onValueChange={setTab}>
            <TabsList>
              <TabsTrigger value={'list'}>Users list</TabsTrigger>
            </TabsList>
          </Tabs>
        </div>
      </div>

      <Outlet />
    </div>
  )
}
