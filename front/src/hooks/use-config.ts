import useConfigStore from "@/store/config.store";

export function useConfig() {
  const { config, setConfig } = useConfigStore()

  return {
    config,
    setConfig,
  }
}