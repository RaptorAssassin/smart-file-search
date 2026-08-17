import { create } from 'zustand'
import { type SearchResult } from '@/bindings/bindings'

interface SearchStore {
  searchQuery: string | null
  files: SearchResult[]
  isSearching: boolean
  setSearchQuery: (query: string) => void
  setFiles: (files: SearchResult[]) => void
  setIsSearching: (isSearching: boolean) => void
}

export const useSearchStore = create<SearchStore>((set) => ({
  searchQuery: null,
  files: [],
  isSearching: false,
  setSearchQuery: (searchQuery: string) => set({ searchQuery }),
  setFiles: (files) => set({ files }),
  setIsSearching: (isSearching) => set({ isSearching }),
}))
