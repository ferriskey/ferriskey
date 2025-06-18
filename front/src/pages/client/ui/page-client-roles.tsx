import { Role } from "@/api/api.interface";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table";

interface PageClientRolesProps {
  roles: Role[];
  isLoading: boolean;
  isError: boolean;
  clientId?: string;
}

export default function PageClientRoles({
  roles,
  isLoading,
  isError,
  clientId
}: PageClientRolesProps) {
  if (isLoading) {
    return <div>Loading roles...</div>;
  }

  if (isError) {
    return <div>Error loading roles.</div>;
  }

  return (
    <div className="container mx-auto py-6">
      <Card>
        <CardHeader>
          <CardTitle>Client Roles - {clientId}</CardTitle>
        </CardHeader>
        <CardContent>
          {roles.length > 0 ? (
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>Role Name</TableHead>
                  <TableHead>Description</TableHead>
                  <TableHead>Created At</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {roles.map((role) => (
                  <TableRow key={role.id}>
                    <TableCell>{role.name}</TableCell>
                    <TableCell>{role.description || "-"}</TableCell>
                    <TableCell>{new Date(role.created_at).toLocaleDateString()}</TableCell>
                  </TableRow>
                ))}
              </TableBody>
            </Table>
          ) : (
            <p>No roles found for this client.</p>
          )}
        </CardContent>
      </Card>
    </div>
  );
} 