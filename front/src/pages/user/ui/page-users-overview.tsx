import { User } from "@/api/api.interface";
import { DataTable } from "@/components/ui/data-table";
import { Edit, ExternalLink, Trash2 } from "lucide-react";
import { useNavigate } from 'react-router';
import { Fragment } from 'react/jsx-runtime';
import { columns } from "../columns/list-user.column";
import EditUserModalFeature from '../feature/edit-user-overview-feature';


export interface PageUsersOverviewOverviewProps {
  isLoading?: boolean
  data: User[]
  realmName: string
  handleDeleteSelected: (items: User[]) => void
}

export default function PageUsersOverview(props: PageUsersOverviewOverviewProps) {
  const navigate = useNavigate()
  const { data, isLoading, handleDeleteSelected } = props

  return (
    <Fragment>
      <DataTable
        data={data}
        columns={columns}
        isLoading={isLoading}
        searchPlaceholder="Rechercher un utilisateur..."
        searchKeys={["username", "id"]}
        enableSelection={true}
        onDeleteSelected={handleDeleteSelected}
        rowActions={[
          {
            label: "Éditer",
            icon: <Edit className="h-4 w-4" />,
            onClick: (user) => navigate(`/realms/${props.realmName}/users/${user.id}/overview`),
          },
          {
            label: "Voir les détails",
            icon: <ExternalLink className="h-4 w-4" />,
            onClick: (user) => console.log("View", user),
          },
          {
            label: "Supprimer",
            icon: <Trash2 className="h-4 w-4" />,
            variant: "destructive",
            onClick: (user) => console.log("Delete", user),
          },
        ]}
      />

      <EditUserModalFeature />
    </Fragment>
  )
}