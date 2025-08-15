export interface UserState {
  isAuthenticated: boolean
  isLoading: boolean
  expiration: number | null
  user: any | null
  setLoading: (value: boolean) => void
  setAuthenticated: (value: boolean) => void
  setUser: (user: any) => void
  setExpiration: (expiration: number | null) => void

}


export interface AuthState {
  accessToken: string | null
  refreshToken: string | null
  setTokens: (accessToken: string, refreshToken: string) => void
}
