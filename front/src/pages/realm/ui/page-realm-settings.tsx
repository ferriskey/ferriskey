import { Heading } from "@/components/ui/heading";
import { Tabs, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { REALM_SETTINGS_URL } from "@/routes/router";
import { Outlet, useNavigate } from "react-router";

interface PageRealmSettingsProps {
  realmName: string
  tab?: string
  setTab?: (value: string) => void
}

export default function PageRealmSettings({ realmName, tab, setTab }: PageRealmSettingsProps) {
  const navigate = useNavigate()
  return (
    <div className="flex flex-col gap-4 p-8">
      <div className="flex flex-col gap-2 border-b pb-4">
        <div className="flex flex-col gap-2">
          <Heading>{realmName}</Heading>
          <p>Realm settings are settings that control the options for users, applications, roles, and groups in the current realm.</p>
        </div>
        <div>
          <Tabs defaultValue={tab} value={tab} onValueChange={(value) => {
            navigate(`${REALM_SETTINGS_URL(realmName)}/${value}`)
            if(setTab) {
              setTab(value || 'general')
            }
          }}>
            <TabsList className="flex items-center gap-4">
              <TabsTrigger value={'general'}>General</TabsTrigger>
              <TabsTrigger value={'login'}>Login</TabsTrigger>
              <TabsTrigger value={'security'}>Security</TabsTrigger>
            </TabsList>
          </Tabs>
        </div>
      </div>
      <Outlet />
    </div>
  )
}
