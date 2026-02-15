import { AuthState } from '@/contracts/states.interface'
import { create } from 'zustand'
import { createJSONStorage, devtools, persist } from 'zustand/middleware'

export const authStore = create<AuthState>()(
  devtools(
    persist(
      (set) => ({
        accessToken: null,
        refreshToken: null,
        idToken: null,
        setTokens: (accessToken: string | null, refreshToken: string | null, idToken: string | null) =>
          set({ accessToken, refreshToken, idToken }),
      }),
      {
        name: 'auth',
        storage: createJSONStorage(() => localStorage)
      }
    )
  )
)
