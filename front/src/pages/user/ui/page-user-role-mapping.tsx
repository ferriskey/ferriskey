import { Role } from "@/api/api.interface";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table";

interface PageUserRoleMappingProps {
  userRoles: Role[];
  isLoading: boolean;
  isError: boolean;
  userId?: string;
}

export default function PageUserRoleMapping({
  userRoles,
  isLoading,
  isError,
  userId
}: PageUserRoleMappingProps) {
  if (isLoading) {
    return <div>Loading user roles...</div>;
  }

  if (isError) {
    return <div>Error loading user roles.</div>;
  }

  return (
    <div className="container mx-auto py-6">
      <Card>
        <CardHeader>
          <CardTitle>User Roles - {userId}</CardTitle>
        </CardHeader>
        <CardContent>
          {userRoles.length > 0 ? (
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>Role Name</TableHead>
                  <TableHead>Description</TableHead>
                  <TableHead>Client</TableHead>
                  <TableHead>Created At</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {userRoles.map((role) => (
                  <TableRow key={role.id}>
                    <TableCell>{role.name}</TableCell>
                    <TableCell>{role.description || "-"}</TableCell>
                    <TableCell>{role.client?.name || "-"}</TableCell>
                    <TableCell>{new Date(role.created_at).toLocaleDateString()}</TableCell>
                  </TableRow>
                ))}
              </TableBody>
            </Table>
          ) : (
            <p>No roles found for this user.</p>
          )}
        </CardContent>
      </Card>
    </div>
  );
} 