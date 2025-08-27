import { DataTable } from "@/components/ui/data-table";
import { columns } from "../columns/list-webhooks.column";
import { useNavigate } from "react-router";

export interface PageRealmSettingsWebhooksProps {

}

export default function PageRealmSettingsWebhooks({ }: PageRealmSettingsWebhooksProps) {
  const navigate = useNavigate()
  return (
    <div>
      <DataTable
        data={[]}
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
  );
}
