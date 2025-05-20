import { UserState } from '@/contracts/states.interface'
import { create } from 'zustand'
import { createJSONStorage, devtools, persist } from 'zustand/middleware'

export const userStore = create<UserState>()(
  devtools(
    persist(
      (set) => ({
        isAuthenticated: false,
        isLoading: true,
        access_token: null,
        refresh_token: null,
        expiration: null,
        setLoading: (value: boolean) => set({ isLoading: value }),
        setAuthenticated: (value: boolean) => set({ isAuthenticated: value }),
        setAuthTokens: (access_token: string, refresh_token: string, expiration: number | null) => set({ access_token, refresh_token, expiration }),
      }),
      {
        name: 'auth',
        storage: createJSONStorage(() => localStorage)
      }
    )
  )
)
