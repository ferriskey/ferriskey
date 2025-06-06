import { Outlet, useLocation, useNavigate, useParams } from 'react-router'
import { useGetUser } from '../../../api/user.api'
import { Heading } from '../../../components/ui/heading'
import { Tabs, TabsList, TabsTrigger } from '../../../components/ui/tabs'
import { UserRouterParams, USERS_URL } from '../../../routes/sub-router/user.router'
import { useEffect, useState } from 'react'
import { REALM_OVERVIEW_URL } from '@/routes/router'
import { Button } from '@/components/ui/button'
import { ArrowLeft } from 'lucide-react'
import BadgeColor, { BadgeColorScheme } from '@/components/ui/badge-color'

export default function UserLayout() {
  const navigate = useNavigate()
  const { realm_name, user_id } = useParams<UserRouterParams>()
  const [defaultValue, setDefaultValue] = useState<string>('')
  const { pathname } = useLocation()

  const { data: user } = useGetUser({
    realm: realm_name,
    userId: user_id
  })

  const handleBack = () => {
    navigate(`${USERS_URL(realm_name)}${REALM_OVERVIEW_URL}`)
  }

  useEffect(() => {
    const pathParts = pathname.split('/')
    const lastPart = pathParts[pathParts.length - 1]

    const validTabs = ['overview', 'credentials', 'role-mapping']
    setDefaultValue(validTabs.includes(lastPart) ? lastPart : 'overview')
  }, [pathname])

  return (
    <div className="p-8">
      <div className="pb-4 mb-4">
        <div className="flex flex-col gap-2 mb-4">
          <div className="flex items-center">
            <Button variant="ghost" size="icon" onClick={handleBack}>
              <ArrowLeft className='h-3 w-3' />
            </Button>

            <span className='text-gray-500 text-sm font-medium'>Back to users</span>
          </div>

          <div className="flex flex-col gap-2">
            <Heading size={3}>{user?.username}</Heading>

            <div className="flex items-center gap-2">
              <span>User ID</span>
              <BadgeColor color={BadgeColorScheme.GRAY}>
                {user?.id}
              </BadgeColor>
            </div>
          </div>
        </div>

        <Tabs
          onValueChange={(value) => navigate(`/realms/${realm_name}/users/${user_id}/${value}`)}
          defaultValue={defaultValue}
          value={defaultValue}
        >
          <div className="flex justify-between items-center w-full border-b pb-4">
            <TabsList>
              <TabsTrigger value="overview">Overview</TabsTrigger>
              <TabsTrigger value="credentials">Credentials</TabsTrigger>
              <TabsTrigger value="role-mapping">Role Mapping</TabsTrigger>
            </TabsList>
          </div>
        </Tabs>
      </div>

     <Outlet />
    </div>
  )
}
