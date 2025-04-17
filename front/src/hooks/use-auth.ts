import { userStore } from '@/store/user.store'

export function useAuth() {
  const { setToken, switchIsAuthenticated } = userStore()

  const setAuthToken = (token: string) => {
    setToken(token)
    switchIsAuthenticated(true)
  }

  return {
    setAuthToken,
  }
}
