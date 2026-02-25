import { Schemas } from '@/api/api.client'
import { useGetSecurityEvents } from '@/api/sea-watch.api'
import { RouterParams } from '@/routes/router'
import { useParams } from 'react-router-dom'
import PageOverview from '../ui/page-overview'

import SecurityEvent = Schemas.SecurityEvent

const mockSecurityEvents: SecurityEvent[] = [
  {
    id: 'sew_evt_01',
    realm_id: 'mock',
    event_type: 'login_failure',
    status: 'failure',
    actor_id: 'john.doe@company.com',
    ip_address: '82.103.44.12',
    user_agent: 'Chrome on Windows',
    timestamp: new Date(Date.now() - 2 * 60 * 1000).toISOString(),
  },
  {
    id: 'sew_evt_02',
    realm_id: 'mock',
    event_type: 'password_reset',
    status: 'success',
    actor_id: 'sarah.wilson@company.com',
    ip_address: '192.168.1.21',
    user_agent: 'Safari on macOS',
    timestamp: new Date(Date.now() - 9 * 60 * 1000).toISOString(),
  },
  {
    id: 'sew_evt_03',
    realm_id: 'mock',
    event_type: 'client_secret_rotated',
    status: 'success',
    actor_id: 'automation@ferriskey',
    resource: 'billing-api',
    ip_address: '10.0.0.6',
    user_agent: 'FerrisKey CLI',
    timestamp: new Date(Date.now() - 18 * 60 * 1000).toISOString(),
  },
  {
    id: 'sew_evt_04',
    realm_id: 'mock',
    event_type: 'role_assigned',
    status: 'success',
    actor_id: 'admin@company.com',
    target_id: 'mike.johnson@company.com',
    target_type: 'user',
    ip_address: '203.44.11.92',
    user_agent: 'Firefox on Ubuntu',
    timestamp: new Date(Date.now() - 32 * 60 * 1000).toISOString(),
  },
  {
    id: 'sew_evt_05',
    realm_id: 'mock',
    event_type: 'login_failure',
    status: 'failure',
    actor_id: 'unknown',
    ip_address: '155.56.70.4',
    user_agent: 'Edge on Windows',
    timestamp: new Date(Date.now() - 46 * 60 * 1000).toISOString(),
  },
]

export default function PageOverviewFeature() {
  const { realm_name } = useParams<RouterParams>()
  const {
    data: responseGetSecurityEvents,
    isLoading,
    isError,
  } = useGetSecurityEvents({ realm: realm_name })

  const events = responseGetSecurityEvents?.data?.length
    ? responseGetSecurityEvents.data
    : mockSecurityEvents

  const isMocked = !responseGetSecurityEvents?.data?.length

  return (
    <PageOverview
      events={events}
      isLoading={isLoading}
      isError={isError}
      realmName={realm_name}
      isMocked={isMocked}
    />
  )
}
