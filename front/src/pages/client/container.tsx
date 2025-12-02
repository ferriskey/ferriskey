import { Tabs, TabsList, TabsTrigger } from '@/components/ui/tabs'
import { Users } from 'lucide-react'
import { useState } from 'react'
import { Outlet } from 'react-router'

export default function Container() {
  const [tab, setTab] = useState('list')

  return (
    <div className='flex flex-col gap-6 p-8'>
      <div className='flex flex-col gap-4 border-b pb-6'>
        <div className='flex items-center gap-4'>
          <div className='flex h-12 w-12 items-center justify-center rounded-lg bg-primary/10'>
            <Users className='h-6 w-6 text-primary' />
          </div>
          <div>
            <h1 className='text-3xl font-semibold tracking-tight'>Clients</h1>
            <p className='text-sm text-muted-foreground'>
              Manage and configure OAuth clients for your applications
            </p>
          </div>
        </div>
        <div className='flex justify-between items-center'>
          <Tabs defaultValue={tab} onValueChange={setTab}>
            <TabsList>
              <TabsTrigger value={'list'}>Clients list</TabsTrigger>
            </TabsList>
          </Tabs>
        </div>
      </div>

      <Outlet />
    </div>
  )
}
