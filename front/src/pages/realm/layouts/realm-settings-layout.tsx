import { RouterParams } from "@/routes/router";
import { useEffect, useState } from "react";
import { useLocation, useParams } from "react-router";
import PageRealmSettings from "../ui/page-realm-settings";

export default function RealmsSettingsLayout() {
  const { realm_name } = useParams<RouterParams>()
  const [tab, setTab] = useState<string>('general');
  const { pathname } = useLocation();

  useEffect(() => {
    const pathParts = pathname.split('/')
    const lastPart = pathParts[pathParts.length - 1]
    const validTabs = ['general', 'login', 'security']

    setTab(validTabs.includes(lastPart) ? lastPart : 'general')
  }, [pathname])

  return <PageRealmSettings realmName={realm_name || "realm"} tab={tab} setTab={setTab} />
}
