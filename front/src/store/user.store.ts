import { UserState } from '@/contracts/states.interface'
import { create } from 'zustand'
import { createJSONStorage, devtools, persist } from 'zustand/middleware'

export const userStore = create<UserState>()(
  devtools(
    persist(
      (set) => ({
        isAuthenticated: false,
        isLoading: true,
        token: null,
        user: null,
        switchIsLoading: (isLoading: boolean) => set({ isLoading }),
        setToken: (token: string) => set({ token }),
        switchIsAuthenticated: (isAuthenticated: boolean) => set({ isAuthenticated }),
      }),
      {
        name: 'user',
        storage: createJSONStorage(() => localStorage),
      }
    )
  )
)
