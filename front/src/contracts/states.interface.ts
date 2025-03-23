export interface UserState {
  isAuthenticated: boolean
  isLoading: boolean
  token: string | null
  user: any | null
}
