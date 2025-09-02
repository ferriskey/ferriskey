import { DataTable } from "@/components/ui/data-table";
import { columns } from "../columns/list-webhooks.column";
import { useNavigate } from "react-router";
import { Schemas } from "@/api/api.client";
import Webhook = Schemas.Webhook

export interface PageRealmSettingsWebhooksProps {
  webhooks: Webhook[]
}

export default function PageRealmSettingsWebhooks({ webhooks }: PageRealmSettingsWebhooksProps) {
  const navigate = useNavigate()
  return (
    <div>
      <DataTable
        data={webhooks}
        columns={columns}
        searchPlaceholder='Find a webhook...'
        searchKeys={['endpoint']}
        createData={{
          label: 'Create Webhook',
          onClick: () => {
            navigate('create')
          }
        }}
      />
    </div>
  )
}
