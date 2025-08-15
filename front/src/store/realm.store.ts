import { create } from "zustand";
import { Schemas } from '@/api/api.client.ts'
import Realm = Schemas.Realm

interface State {
  userRealms: Realm[]
}

interface Actions {
  setUserRealms: (realms: Realm[]) => void
}

const useRealmStore = create<State & Actions>((set) => ({
  userRealms: [],
  setUserRealms: (realms: Realm[]) => set({ userRealms: realms }),
}))

export default useRealmStore
