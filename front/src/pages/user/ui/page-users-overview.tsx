import { User } from "@/api/api.interface";
import { DataTable } from "@/components/ui/data-table";
import { Edit, ExternalLink, Trash2 } from "lucide-react";
import { columns } from "../columns/list-client.column";


export interface PageUsersOverviewOverviewProps {
  isLoading?: boolean
  data: User[]
  realmName: string
  handleDeleteSelected: (items: User[]) => void
}

export default function PageUsersOverview({ data, isLoading, handleDeleteSelected }: PageUsersOverviewOverviewProps) {
  return (
    <div>
       <DataTable
        data={data}
        columns={columns}
        isLoading={isLoading}
        searchPlaceholder="Rechercher un client..."
        searchKeys={["username", "client_id"]}
        enableSelection={true}
        onDeleteSelected={handleDeleteSelected}
        rowActions={[
          {
            label: "Éditer",
            icon: <Edit className="h-4 w-4" />,
            onClick: (client) => console.log("Edit", client),
          },
          {
            label: "Voir les détails",
            icon: <ExternalLink className="h-4 w-4" />,
            onClick: (client) => console.log("View", client),
          },
          {
            label: "Supprimer",
            icon: <Trash2 className="h-4 w-4" />,
            variant: "destructive",
            onClick: (client) => console.log("Delete", client),
          },
        ]}
      />
    </div>
  )
}