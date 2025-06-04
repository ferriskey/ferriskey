import { CredentialOverview } from "@/api/api.interface";
import { DataTable } from "@/components/ui/data-table";
import { columnsUserCredential } from "../columns/list-user-credential.column";

export interface PageCredentialsProps {
  credentials: CredentialOverview[]
}

export default function PageCredentials({ credentials }: PageCredentialsProps) {

  return (
    <DataTable 
      data={credentials}
      columns={columnsUserCredential}
      searchPlaceholder="Search a credential..."
      enableSelection={true}
    />
  )
}