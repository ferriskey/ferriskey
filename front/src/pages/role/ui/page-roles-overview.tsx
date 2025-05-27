import { Role } from "@/api/api.interface";
import { DataTable } from "@/components/ui/data-table";
import { Edit, ExternalLink, Trash2, Settings } from "lucide-react";
import { columns } from "../columns/list-client.column";
import CreateRoleModalFeature from '../feature/create-role-modal-feature';
import { Heading } from "@/components/ui/heading";
import { useNavigate } from "react-router-dom";

export interface PageRolesOverviewProps {
  isLoading?: boolean
  data: Role[]
  realmName: string
  handleDeleteSelected: (items: Role[]) => void
  handleClickRow: (roleId: string) => void
}

export default function PageRolesOverview({ data, isLoading, realmName, handleDeleteSelected, handleClickRow }: PageRolesOverviewProps) {
  const navigate = useNavigate();

  const handleViewSettings = (role: Role) => {
    navigate(`/realms/${realmName}/roles/${role.id}/settings`);
  };

  return (
    <div className="space-y-6 p-6">
      <div className="flex items-center justify-between">
        <div>
          <Heading>Roles</Heading>
          <p className="text-muted-foreground">
            Manage roles in {realmName}
          </p>
        </div>
        <CreateRoleModalFeature />
      </div>

      <DataTable
        data={data}
        columns={columns}
        isLoading={isLoading}
        searchPlaceholder="Search a client..."
        searchKeys={["name", "client_id"]}
        enableSelection={true}
        onDeleteSelected={handleDeleteSelected}
        onRowClick={(role) => {
          handleClickRow(role.id)
        }}
        rowActions={[
          {
            label: "Paramètres",
            icon: <Settings className="h-4 w-4" />,
            onClick: handleViewSettings,
          },
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