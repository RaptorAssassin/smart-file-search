import { create } from 'zustand'

interface SearchStore {
  searchQuery: string | null
  setSearchQuery: (query: string) => void
}

export const useSearchStore = create<SearchStore>((set) => ({
  searchQuery: null,
  setSearchQuery: (searchQuery: string) => set({ searchQuery }),
}))
