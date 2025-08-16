import { RouterParams } from "@/routes/router";
import PageRealmSettings from "../ui/page-realm-settings";
import { useParams } from "react-router";

export default function PageRealmSettingsFeature() {
  const { realm_name } = useParams<RouterParams>()
  return <PageRealmSettings realm_name={realm_name} />
}
