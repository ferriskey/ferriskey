import { useGetRole } from "@/api/role.api";
import BadgeColor, { BadgeColorScheme } from "@/components/ui/badge-color";
import { Button } from "@/components/ui/button";
import { Heading } from "@/components/ui/heading";
import { Tabs, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { RouterParams } from "@/routes/router";
import { ROLE_OVERVIEW_URL, ROLE_URL, ROLES_URL } from "@/routes/sub-router/role.router";
import { ArrowLeft } from "lucide-react";
import { Outlet, useNavigate, useParams } from "react-router";

export default function RoleLayout() {
  const { realm_name, role_id } = useParams<RouterParams>()
  const navigate = useNavigate()

  const { data: role } = useGetRole({
    realm: realm_name || "master",
    roleId: role_id,  
  })

  const handleBack = () => {
    navigate(`${ROLES_URL(realm_name)}${ROLE_OVERVIEW_URL}`)
  }

  const handleTabChange = (value: string) => {
    navigate(`${ROLE_URL(realm_name, role_id)}/${value}`)
  }

  return (
    <div className="p-4">
      <div className="pb-4 mb-4">
        <div className="flex flex-col gap-2 mb-4">
          <div className="flex items-center">

            <Button variant="ghost" size="icon" onClick={handleBack}>
              <ArrowLeft className="h-3 w-3" />
              
            </Button>
            <span className="text-gray-500 text-sm font-medium">Back to roles</span>
          </div>
          <div className="flex flex-col gap-2">
            <Heading size={3}>{role?.name}</Heading>

            <div className="flex items-center gap-2">
              <span>Role ID</span>
              <BadgeColor color={BadgeColorScheme.GRAY}>
                {role?.id}
              </BadgeColor>
            </div>
          </div>
        </div>

        <div>
          <Tabs defaultValue="settings" onValueChange={handleTabChange}>
            <TabsList className="flex items-center gap-4">
              <TabsTrigger value={"settings"}>Settings</TabsTrigger>
              <TabsTrigger value={"permissions"}>Permissions</TabsTrigger>
              <TabsTrigger value={"users"}>Users in role</TabsTrigger>
            </TabsList>
          </Tabs>
        </div>
      </div>

      <Outlet />
    </div>
  )
}