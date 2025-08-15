import { Heading } from '@/components/ui/heading'
import { REALM_URL, RouterParams } from '@/routes/router'
import { OVERVIEW_URL } from '@/routes/sub-router/client.router'
import { ROLE_OVERVIEW_URL } from '@/routes/sub-router/role.router'
import { USER_OVERVIEW_URL } from '@/routes/sub-router/user.router'
import { Pyramid, ShieldUser, Users } from 'lucide-react'
import { useNavigate, useParams } from 'react-router'

export interface PageHomeProps {}

const items = [
  {
    title: 'Client',
    icon: Pyramid,
    url: `/clients${OVERVIEW_URL}`,
  },
  {
    title: 'User',
    icon: Users,
    url: `/users${USER_OVERVIEW_URL}`,
  },
  {
    title: 'Role',
    icon: ShieldUser,
    url: `/roles${ROLE_OVERVIEW_URL}`,
  },
]

export default function PageHome({}: PageHomeProps) {
  const { realm_name } = useParams<RouterParams>()
  const navigate = useNavigate()

  const handleClick = (url: string) => {
    if (!realm_name) return

    navigate(`${REALM_URL(realm_name)}${url}`)
  }

  return (
    <div className='flex flex-col gap-6 p-6 md:p-10 container mx-auto'>
      <div className='flex flex-col gap-6'>
        <div className='flex items-center gap-3'>
          <img src='/logo_ferriskey.png' alt='FerrisKey Logo' className='h-12' />
          <Heading weight='medium'>Welcome</Heading>
        </div>
        <span className='text-md text-gray-700'>
          You're working in <span className='text-primary'>{realm_name}</span> realm
        </span>

        <div>
          <Heading size={3} weight='light' className='mt-4'>
            Quick access
          </Heading>

          <div className='grid sm:grid-cols-2 lg:grid-cols-4 gap-4 mt-4'>
            {items.map((item, index) => (
              <div
                onClick={() => handleClick(item.url)}
                key={index}
                className='border rounded-md p-3 py-6 hover:shadow-md hover:cursor-pointer shadow-primary/10'
              >
                <div className='flex items-center gap-3'>
                  <item.icon className='size-5' />

                  <span className='text-sm'>{item.title}</span>
                </div>
              </div>
            ))}
          </div>
        </div>
      </div>
    </div>
  )
}
