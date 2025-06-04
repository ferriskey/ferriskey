import { Role } from "@/api/api.interface";
import { DataTable } from "@/components/ui/data-table";
import { Edit, ExternalLink, Trash2, Settings } from "lucide-react";
import { columns } from "../columns/list-client.column";
import { Heading } from "@/components/ui/heading";
import { useNavigate } from "react-router-dom";
import { Button } from "@/components/ui/button";
import { ROLE_CREATE_URL, ROLES_URL } from "@/routes/sub-router/role.router";

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

  const handleCreateRole = () => {
    navigate(`${ROLES_URL(realmName)}${ROLE_CREATE_URL}`)
  }

  return (
    <div className="space-y-6 p-6">
      <div className="flex items-center justify-between">
        <div>
          <Heading>Roles</Heading>
          <p className="text-muted-foreground">
            Manage roles in {realmName}
          </p>
        </div>
        <Button variant="outline" onClick={handleCreateRole}>
          Create Role
        </Button>
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