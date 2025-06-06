import { User } from "@/api/api.interface";
import { DataTable } from "@/components/ui/data-table";
import { Edit, ExternalLink, Trash2 } from "lucide-react";
import { useNavigate } from 'react-router';
import { Fragment } from 'react/jsx-runtime';
import { columns } from "../columns/list-user.column";
import EditUserModalFeature from '../feature/page-user-overview-feature.tsx';


export interface PageUsersOverviewOverviewProps {
  isLoading?: boolean
  data: User[]
  realmName: string
  handleDeleteSelected: (items: User[]) => void
  handleClickRow: (userId: string) => void
}

export default function PageUsersOverview({
  isLoading,
  data,
  realmName,
  handleClickRow,
  handleDeleteSelected
}: PageUsersOverviewOverviewProps) {
  const navigate = useNavigate()

  return (
    <Fragment>
      <DataTable
        data={data}
        columns={columns}
        isLoading={isLoading}
        searchPlaceholder="Search a user..."
        searchKeys={["username", "id"]}
        enableSelection={true}
        onRowClick={(user) => {
          handleClickRow(user.id)
        }}
        onDeleteSelected={handleDeleteSelected}
        rowActions={[
          {
            label: "Edit",
            icon: <Edit className="h-4 w-4" />,
            onClick: (user) => navigate(`/realms/${realmName}/users/${user.id}/overview`),
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
