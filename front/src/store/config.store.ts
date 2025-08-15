import { GetConfigResponse } from '@/api/api.interface'
import { create } from 'zustand'

interface State {
  config: GetConfigResponse | null
}

interface Actions {
  setConfig: (config: GetConfigResponse) => void
}

const useConfigStore = create<State & Actions>((set) => ({
  config: null,
  setConfig: (config: GetConfigResponse) => set({ config }),
}))

export default useConfigStore
