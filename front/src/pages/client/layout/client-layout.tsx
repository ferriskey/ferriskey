import { useGetClient } from "@/api/client.api";
import BadgeColor, { BadgeColorScheme } from "@/components/ui/badge-color";
import { Heading } from "@/components/ui/heading";
import { Tabs, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { Outlet, useParams } from "react-router";

export default function ClientLayout() {
  const { realm_name, client_id } = useParams()

  const { data } = useGetClient({
    realm: realm_name ?? 'master',
    clientId: client_id,
  })

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
          <Tabs defaultValue="settings">
            <TabsList className="flex items-center gap-4">
              <TabsTrigger value={"settings"}>Settings</TabsTrigger>
              <TabsTrigger disabled value={"keys"}>Keys</TabsTrigger>
              <TabsTrigger disabled value={"credentials"}>Credentials</TabsTrigger>
              <TabsTrigger disabled value={"roles"}>Roles</TabsTrigger>
              <TabsTrigger disabled value={"client-scopes"}>Client scopes</TabsTrigger>
            </TabsList>
          </Tabs>
        </div>
      </div>
      

      <Outlet />
    </div>
  )
}