import userStore from '@/store/user.store.ts'

export function useUser() {
  const { user } = userStore()

  return {
    user
  }
}
