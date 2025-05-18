export interface UserState {
  isAuthenticated: boolean
  isLoading: boolean
  token: string | null
  user: any | null
  switchIsLoading: (isLoading: boolean) => void
  setToken: (token: string) => void
  switchIsAuthenticated: (isAuthenticated: boolean) => void
  access_token?: string | null
  refresh_token?: string | null
  expiration?: number | null
  setAuthTokens?: (access_token: string, refresh_token: string, expiration: number | null) => void
}
