import { CredentialOverview } from "@/api/api.interface";
import { ColumnDef } from "@/components/ui/data-table";

export const columnsUserCredential: ColumnDef<CredentialOverview>[] = [
  {
    id: "type",
    header: "Type",
    cell: (credential) => {
      return (
        <span className="font-medium">{credential.credential_type}</span>
      );
    }
  },
  {
    id: "user_label",
    header: "User Label",
    cell: (credential) => {
      return (
        <span className="font-medium">{credential.user_label ?? "N/A"}</span>
      );
    }
  },
  {
    id: "created_at",
    header: "Created At",
    cell: (credential) => {
      return (
        <span className="text-xs text-muted-foreground">
          {new Date(credential.created_at).toLocaleDateString("en-US", {
            year: 'numeric',
            month: '2-digit',
            day: '2-digit',
            hour: '2-digit',
            minute: '2-digit'
          })}
        </span>
      );
    }
  },
]
