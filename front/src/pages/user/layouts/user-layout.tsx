import { Route, Routes, useNavigate, useParams } from 'react-router'
import { useGetUser } from '../../../api/user.api'
import { Heading } from '../../../components/ui/heading'
import { Tabs, TabsList, TabsTrigger } from '../../../components/ui/tabs'
import { UserRouterParams } from '../../../routes/sub-router/user.router'
import EditUserOverviewFeature from '../feature/edit-user-overview-feature'

export default function UserLayout() {
  const navigate = useNavigate()
  const { realm_name, user_id } = useParams<UserRouterParams>()

  const { data: user } = useGetUser({
    realm: realm_name,
    userId: user_id
  })

  return (
    <div className="flex flex-col gap-4 p-8">
      <div className="flex flex-col gap-2">
        <Heading>{user?.username}</Heading>
        <p>Manage user in {realm_name}</p>
      </div>
      <Tabs
        onValueChange={(value) => navigate(`/realms/${realm_name}/users/${user_id}/${value}`)}
        defaultValue="overview"
      >
        <div className="flex justify-between items-center w-full border-b pb-4">
          <TabsList>
            <TabsTrigger value="overview">Overview</TabsTrigger>
            <TabsTrigger value="credentials">Credentials</TabsTrigger>
            <TabsTrigger value="role-mapping">Role Mapping</TabsTrigger>
          </TabsList>
        </div>
      </Tabs>

      <Routes>
        <Route path="overview" element={<EditUserOverviewFeature />} />
        <Route path="credentials" element={<p>credentials</p>} />
        <Route path="role-mapping" element={<p>role-mapping</p>} />
      </Routes>
    </div>
  )
}