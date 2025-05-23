import { UserState } from '@/contracts/states.interface'
import { create } from 'zustand'

const userStore = create<UserState>((set) => ({
  expiration: null,
  isAuthenticated: false,
  isLoading: true,
  user: null,
  setAuthenticated: (value: boolean) => set({ isAuthenticated: value }),
  setLoading: (value: boolean) => set({ isLoading: value }),
  setUser: (user: any) => set({ user }),
  setExpiration: (expiration: number | null) => set({ expiration }),
}))

export default userStore