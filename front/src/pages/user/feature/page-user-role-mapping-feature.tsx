import { useParams } from "react-router-dom";
import { useGetRoles } from "@/api/role.api.ts";

export default function PageUserRoleMappingFeature() {
  const { user_id } = useParams();
  // For now, assuming realm is hardcoded or comes from another context. We might need to fetch this dynamically.
  // For demonstration, let's assume a placeholder realm for now.
  const realm = "master"; // Placeholder, needs to be dynamic

  const { data: roles, isLoading, isError } = useGetRoles({ realm });

  if (isLoading) {
    return <div>Loading roles...</div>;
  }

  if (isError) {
    return <div>Error loading roles.</div>;
  }

  return (
    <div>
      <h2>Roles for User ID: {user_id}</h2>
      {roles?.data && roles.data.length > 0 ? (
        <ul>
          {roles.data.map((role) => (
            <li key={role.id}>{role.name}</li>
          ))}
        </ul>
      ) : (
        <p>No roles found for this user.</p>
      )}
    </div>
  );
} 