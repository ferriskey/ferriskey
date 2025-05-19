import { Client } from "@/api/api.interface"
import { useGetClients } from "@/api/client.api"
import { Button } from "@/components/ui/button"
import { ColumnDef, DataTable } from "@/components/ui/data-table"
import { RouterParams } from "@/routes/router"
import { Edit, ExternalLink, Plus, Trash2 } from "lucide-react"
import { useParams } from "react-router"

export default function PageClientsOverviewFeature() {
  const { realm_name } = useParams<RouterParams>()
  const { data, isLoading } = useGetClients({ realm: 'master' })

  const columns: ColumnDef<Client>[] = [
    {
      id: "name",
      header: "Utilisateur",
      cell: (client) => (
        <div className="flex items-center gap-3">
          <div className="h-8 w-8 rounded-full bg-purple-100 flex items-center justify-center">
            <span className="text-xs font-medium text-purple-600">{client.name?.[0]?.toUpperCase() || 'C'}</span>
          </div>
          <div>
            <div className="font-medium">{client.name}</div>
            <div className="text-xs text-muted-foreground">{client.client_id}</div>
          </div>
        </div>
      ),
    },
    {
      id: "type",
      header: "Type",
      cell: (client) => (
        <span className="text-sm px-2 py-0.5 rounded-md bg-slate-100">
          {client.public_client ? "Public" : "Confidentiel"}
        </span>
      ),
    },
    {
      id: "status",
      header: "Statut",
      cell: (client) => (
        <div className="flex items-center">
          <span 
            className={`h-2 w-2 rounded-full ${client.enabled ? "bg-emerald-500" : "bg-red-500"} mr-2`}
          ></span>
          <span>{client.enabled ? "Activé" : "Désactivé"}</span>
        </div>
      ),
    },
  ]

  const handleDeleteSelected = (items: Client[]) => {
    console.log("Deleting", items);
    // Logique de suppression en lot
  };

  return (
    <div className="space-y-6 p-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold tracking-tight">Clients</h1>
          <p className="text-muted-foreground">
            Gérez les clients et leurs configurations dans {realm_name || "master"}
          </p>
        </div>
        <Button className="flex items-center gap-2">
          <Plus className="h-4 w-4" />
          <span>Nouveau Client</span>
        </Button>
      </div>

      <DataTable
        data={data?.data || []}
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
