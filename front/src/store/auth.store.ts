import { AuthState } from "@/contracts/states.interface";
import { create } from "zustand";
import { createJSONStorage, devtools, persist } from "zustand/middleware";

export const authStore = create<AuthState>()(
  devtools(
    persist(
      (set) => ({
        accessToken: null,
        refreshToken: null,
        setTokens: (accessToken: string, refreshToken: string) => set({ accessToken, refreshToken }),
      }),
      {
        name: 'auth',
        storage: createJSONStorage(() => localStorage)
      }
    )
  )
)