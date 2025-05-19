import { User } from "@/api/api.interface";
import { DataTable } from "@/components/ui/data-table";
import { Edit, ExternalLink, Trash2 } from "lucide-react";
import { columns } from "../columns/list-client.column";
import CreateClientModalFeature from "../feature/create-client-modal-feature";

export interface PageUsersOverviewOverviewProps {
  isLoading?: boolean
  data: User[]
  realmName: string
  handleDeleteSelected: (items: User[]) => void
}

export default function PageUsersOverview({ data, isLoading, realmName, handleDeleteSelected }: PageUsersOverviewOverviewProps) {
  return (
    <div className="space-y-6 p-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold tracking-tight">Users</h1>
          <p className="text-muted-foreground">
            Gérez les utilisateurs dans {realmName}
          </p>
        </div>
        <CreateClientModalFeature />
      </div>

      <DataTable
        data={data}
        columns={columns}
        isLoading={isLoading}
        searchPlaceholder="Rechercher un client..."
        searchKeys={["name", "client_id"]}
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