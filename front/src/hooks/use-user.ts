import { userStore } from '@/store/user.store'
import { useEffect } from 'react'

const useUser = () => {
  const { isAuthenticated, isLoading, token, user, switchIsLoading } = userStore()

  async function login(email: string, password: string) {}

  useEffect(() => {
    if (!isAuthenticated && isLoading && !token) {
      switchIsLoading(false)
    }
  }, [isAuthenticated, isLoading, token, switchIsLoading])

  return {
    isAuthenticated,
    isLoading,
    token,
    user,
  }
}
export default useUser
