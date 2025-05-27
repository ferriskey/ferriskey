import { useParams } from "react-router-dom";
import { useGetRole } from "@/api/role.api";
import PageRoleSettings from "../ui/page-role-settings";
import { RouterParams } from "@/routes/router";


export default function PageRoleSettingsFeature() {
  const { realm_name, role_id } = useParams<RouterParams>();
  
  const { data: role, isLoading } = useGetRole({
    realm: realm_name || "master",
    roleId: role_id,
  });

  return (
    <PageRoleSettings
      role={role}
      isLoading={isLoading}
      realmName={realm_name || "master"}
    />
  );
}