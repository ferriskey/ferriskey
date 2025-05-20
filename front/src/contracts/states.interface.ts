export interface UserState {
  isAuthenticated: boolean
  isLoading: boolean
  access_token: string | null
  refresh_token: string | null
  expiration: number | null
  setLoading: (value: boolean) => void
  setAuthenticated: (value: boolean) => void
  setAuthTokens: (access_token: string, refresh_token: string, expiration: number | null) => void
}
