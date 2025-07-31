import { CredentialOverview } from "@/api/core.interface";
import { DataTable } from "@/components/ui/data-table";
import { columnsUserCredential } from "../columns/list-user-credential.column";
import SetPasswordFeature from "../feature/modals/set-password-feature";
import { Trash2 } from 'lucide-react'

export interface PageCredentialsProps {
  credentials: CredentialOverview[]
  handleDeleteUserCredential: (credentialId: string) => void
}

export default function PageCredentials({ credentials, handleDeleteUserCredential }: PageCredentialsProps) {
  return (
    <DataTable
      data={credentials}
      columns={columnsUserCredential}
      searchPlaceholder="Search a credential..."
      enableSelection={true}
      emptyState={<EmptyCredential />}
      onDeleteSelected={() => {
        credentials.forEach(c => {
          handleDeleteUserCredential(c.id)
        })
      }}
      rowActions={[
        {
          label: 'Delete',
          icon: <Trash2 className="w-4 h-4" />,
          variant: "destructive",
          onClick: (credential) => {
            handleDeleteUserCredential(credential.id)
          }
        }
      ]}
    />
  )
}

function EmptyCredential() {
  return (
    <div className="text-center flex flex-col gap-3 py-8">
      <img src="/event-placeholder-light.svg" alt="No credentials" className="mx-auto mb-4 w-40" />
      <div className="">
        <h2 className="text-lg font-semibold text-neutral-600">No Credentials Found</h2>
        <p className="text-muted-foreground">You have no credentials available.</p>
      </div>
      <div>
        <SetPasswordFeature />
      </div>
    </div>
  );
}
