import { useQuery } from '@tanstack/react-query'

export interface PublicPasswordPolicy {
  min_length: number
  require_uppercase: boolean
  require_lowercase: boolean
  require_number: boolean
  require_special: boolean
}

export const DEFAULT_PASSWORD_POLICY: PublicPasswordPolicy = {
  min_length: 8,
  require_uppercase: false,
  require_lowercase: false,
  require_number: false,
  require_special: false,
}

async function fetchPublicPasswordPolicy(realmName: string): Promise<PublicPasswordPolicy> {
  const base = (window.apiUrl ?? '').replace(/\/$/, '')
  const url = `${base}/realms/${encodeURIComponent(realmName)}/password-policy/public`
  const response = await fetch(url, {
    method: 'GET',
    headers: { 'Content-Type': 'application/json' },
    credentials: 'include',
  })

  if (!response.ok) {
    throw new Error(`Failed to fetch password policy: HTTP ${response.status}`)
  }

  return response.json() as Promise<PublicPasswordPolicy>
}

export function usePublicPasswordPolicy(realmName: string | undefined) {
  return useQuery<PublicPasswordPolicy>({
    queryKey: ['public-password-policy', realmName],
    queryFn: () => fetchPublicPasswordPolicy(realmName!),
    enabled: !!realmName,
    retry: false,
    placeholderData: DEFAULT_PASSWORD_POLICY,
  })
}
