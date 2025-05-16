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
        access_token: null,
        refresh_token: null,
        expiration: null,
        switchIsLoading: (isLoading: boolean) => set({ isLoading }),
        setToken: (token: string) => set({ token }),
        switchIsAuthenticated: (isAuthenticated: boolean) => set({ isAuthenticated }),
        setAuthTokens: (access_token: string, refresh_token: string, expiration: number | null) => set({ access_token, refresh_token, expiration })
      }),
      {
        name: 'user',
        storage: createJSONStorage(() => localStorage)
      }
    )
  )
)
