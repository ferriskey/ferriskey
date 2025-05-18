import { Realm } from "@/api/api.interface";
import { create } from "zustand";

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