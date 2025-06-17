import { useGetClient } from "@/api/client.api";
import BadgeColor, { BadgeColorScheme } from "@/components/ui/badge-color";
import { Heading } from "@/components/ui/heading";
import { Tabs, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { RouterParams } from "@/routes/router";
import { CLIENT_URL } from "@/routes/sub-router/client.router";
import { useEffect, useState } from "react";
import { Outlet, useLocation, useNavigate, useParams } from "react-router";

export default function ClientLayout() {
  const { realm_name, client_id } = useParams<RouterParams>()
  const [defaultValue, setDefaultValue] = useState<string>("")
  const { pathname } = useLocation()

  useEffect(() => {
    const pathParts = pathname.split('/')
    const lastPart = pathParts[pathParts.length - 1]

    const validTabs = ['credentials', 'roles', 'client-scopes']
    setDefaultValue(validTabs.includes(lastPart) ? lastPart : 'settings')
  }, [pathname])

  const navigate = useNavigate()

  const { data } = useGetClient({
    realm: realm_name ?? 'master',
    clientId: client_id,
  })

  const handleTabChange = (value: string) => {
    setDefaultValue(value)
    navigate(`${CLIENT_URL(realm_name, client_id)}/${value}`)
  }

  return (
    <div className="p-4">
      
      <div className="border-b pb-4 mb-4">
        <div className="flex flex-col gap-2 mb-4">
          <div className="flex items-center gap-4">
            <Heading size={3}>{data?.client_id}</Heading>
            <div>
              <BadgeColor color={BadgeColorScheme.PRIMARY}>
                {data?.protocol}
              </BadgeColor>
            </div>
          </div>
          <p className="text-sm text-gray-500">
            Clients are applications and services that can request authentication of a user.
          </p>
        </div>

        <div>
          <Tabs value={defaultValue} defaultValue={defaultValue} onValueChange={handleTabChange}>
            <TabsList className="flex items-center gap-4">
              <TabsTrigger value={"settings"}>Settings</TabsTrigger>
              <TabsTrigger value={"credentials"}>Credentials</TabsTrigger>
              <TabsTrigger value={"roles"}>Roles</TabsTrigger>
              <TabsTrigger disabled value={"client-scopes"}>Client scopes</TabsTrigger>
            </TabsList>
          </Tabs>
        </div>
      </div>
      

      <Outlet />
    </div>
  )
}