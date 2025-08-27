import { useGetWebhooks } from "@/api/webhook.api";
import PageRealmSettingsWebhooks from "../ui/page-realm-settings-webhooks";
import { useParams } from "react-router";
import { RouterParams } from "@/routes/router";

export default function PageRealmSettingsWebhooksFeature() {
  const { realm_name } = useParams<RouterParams>()
  const { data: webhooks } = useGetWebhooks({ realm: realm_name })
  return (
    <PageRealmSettingsWebhooks />
  )
}
