import { create } from 'zustand';
import { type Config } from '@/bindings/bindings';

interface ConfigStore {
  config: Config | null;

  setConfig: (config: Config) => void;
}

export const useConfigStore = create<ConfigStore>((set) => ({
  config: null,

  setConfig: (config: Config) => set({ config }),
}));
