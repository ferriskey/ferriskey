import { useParams } from 'react-router';
import { useGetClientRoles } from '@/api/client.api';
import { ClientRouterParams } from '@/routes/sub-router/client.router';
import PageClientRoles from '../ui/page-client-roles';

export default function PageClientRolesFeature() {
  const { realm_name, client_id } = useParams<ClientRouterParams>();

  const { data: roles, isLoading, isError } = useGetClientRoles({ 
    realm: realm_name || "master", 
    clientId: client_id 
  });

  return (
    <PageClientRoles
      roles={roles?.data || []}
      isLoading={isLoading}
      isError={isError}
      clientId={client_id}
    />
  );
} 