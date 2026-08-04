import { create } from 'zustand'

interface UIStore {
  settingsOpen: boolean
  setSettingsOpen: (open: boolean) => void

  debugOpen: boolean
  setDebugOpen: (open: boolean) => void

  searchBarFocused: boolean
  setSearchBarFocused: (focused: boolean) => void
}

export const useUIStore = create<UIStore>((set) => ({
  settingsOpen: false,
  setSettingsOpen: (open: boolean) => set({ settingsOpen: open }),

  debugOpen: false,
  setDebugOpen: (open: boolean) => set({ debugOpen: open }),

  searchBarFocused: false,
  setSearchBarFocused: (focused: boolean) => set({ searchBarFocused: focused }),
}))
