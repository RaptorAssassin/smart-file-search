import { create } from 'zustand'
import { type Config } from '@/bindings/bindings'
import { commands } from '@/bindings/bindings'

interface ConfigStore {
  config: Config | null

  loadConfig: () => Promise<void>

  setConfig: (config: Config) => void

  saveConfig: () => void
}

export const useConfigStore = create<ConfigStore>((set, get) => ({
  config: null,

  loadConfig: async () => {
    const config = await commands.getConfig()
    set({ config })
  },

  setConfig: (config: Config) => set({ config }),

  saveConfig: async () => {
    const config = get().config
    if (!config) return

    await commands.saveConfig(config)
  },
}))
