export interface UserState {
  isAuthenticated: boolean
  isLoading: boolean
  switchIsLoading: (isLoading: boolean) => void
  switchIsAuthenticated: (isAuthenticated: boolean) => void
  access_token?: string | null
  refresh_token?: string | null
  expiration?: number | null
  setAuthTokens?: (access_token: string, refresh_token: string, expiration: number | null) => void
}
