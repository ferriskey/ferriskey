import type { AuthState } from '@/contracts/states.interface'
import { create } from 'zustand'
import { createJSONStorage, devtools, persist } from 'zustand/middleware'

const AUTH_STORAGE_KEY = 'auth'

export function removeLegacyPersistedTokens(legacyStorage: () => Storage) {
  try {
    legacyStorage().removeItem(AUTH_STORAGE_KEY)
  } catch {
    return
  }
}

removeLegacyPersistedTokens(() => localStorage)

export const authStore = create<AuthState>()(
  devtools(
    persist(
      (set) => ({
        accessToken: null,
        refreshToken: null,
        idToken: null,
        sessionRealm: null,
        setTokens: (accessToken: string | null, refreshToken: string | null, idToken: string | null) =>
          set({ accessToken, refreshToken, idToken }),
        setSessionRealm: (sessionRealm: string | null) => set({ sessionRealm }),
      }),
      {
        name: AUTH_STORAGE_KEY,
        storage: createJSONStorage(() => sessionStorage)
      }
    )
  )
)
