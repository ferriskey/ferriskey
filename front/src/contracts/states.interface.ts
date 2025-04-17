export interface UserState {
  isAuthenticated: boolean
  isLoading: boolean
  token: string | null
  user: any | null
  switchIsLoading: (isLoading: boolean) => void
  setToken: (token: string) => void
  switchIsAuthenticated: (isAuthenticated: boolean) => void
}
