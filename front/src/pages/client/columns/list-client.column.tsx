import { Client } from "@/api/api.interface";
import { ColumnDef } from "@/components/ui/data-table";

export const columns: ColumnDef<Client>[] = [
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