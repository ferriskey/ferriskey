export interface IUser {
  avatar: string
  preferred_username: string
  email: string
  name: string
}

export interface UserState {
  isAuthenticated: boolean
  isLoading: boolean
  expiration: number | null
  user: IUser | null
  setLoading: (value: boolean) => void
  setAuthenticated: (value: boolean) => void
  setUser: (user: IUser) => void
  setExpiration: (expiration: number | null) => void
}

export interface AuthState {
  accessToken: string | null
  refreshToken: string | null
  setTokens: (accessToken: string, refreshToken: string) => void
}
