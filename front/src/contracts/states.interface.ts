/** One entry in the `organizations` JWT claim, produced by oidc-organization-membership-mapper. */
export interface OrganizationClaim {
  id: string
  name: string
  alias: string
  /** Present when the mapper has `include.domain = true`. */
  domain?: string | null
  /** Present when the mapper has `include.attributes = true`. */
  attributes?: Record<string, string>
}

export interface IUser {
  avatar: string
  preferred_username: string
  email: string
  name: string
  /**
   * Organizations the user belongs to, keyed by alias.
   * e.g. `{ acme: { id, name, alias, ... } }`
   * Empty object when the claim is absent from the token.
   */
  organizations: Record<string, OrganizationClaim>
}

export interface UserState {
  isAuthenticated: boolean
  isLoading: boolean
  expiration: number | null
  user: IUser | null
  setLoading: (value: boolean) => void
  setAuthenticated: (value: boolean) => void
  setUser: (user: IUser | null) => void
  setExpiration: (expiration: number | null) => void
}

export interface AuthState {
  accessToken: string | null
  refreshToken: string | null
  idToken: string | null
  setTokens: (
    accessToken: string | null,
    refreshToken: string | null,
    idToken: string | null
  ) => void
}
