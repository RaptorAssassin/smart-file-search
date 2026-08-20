import { create } from 'zustand'
import {
  type SearchFilters,
  type SearchFilterOptions,
  type SearchResult,
} from '@/bindings/bindings'

interface SearchStore {
  searchQuery: string | null
  files: SearchResult[]
  isSearching: boolean
  filters: SearchFilters
  filterOptions: SearchFilterOptions | null
  unavailable: string[]
  searchError: string | null
  selectedFile: SearchResult | null
  activeIndex: number | null
  setSearchQuery: (query: string) => void
  setFiles: (files: SearchResult[]) => void
  setIsSearching: (isSearching: boolean) => void
  setFilters: (filters: SearchFilters) => void
  setFilterOptions: (options: SearchFilterOptions) => void
  setUnavailable: (unavailable: string[]) => void
  setSearchError: (error: string | null) => void
  setSelectedFile: (file: SearchResult | null) => void
  setActiveIndex: (index: number | null) => void
}

export const useSearchStore = create<SearchStore>((set) => ({
  searchQuery: null,
  files: [],
  isSearching: false,
  filters: {
    extensions: [],
    min_size: null,
    max_size: null,
    modified_after: null,
    modified_before: null,
    created_after: null,
    created_before: null,
  },
  filterOptions: null,
  unavailable: [],
  searchError: null,
  selectedFile: null,
  activeIndex: null,
  setSearchQuery: (searchQuery: string) => set({ searchQuery }),
  setFiles: (files) => set({ files }),
  setIsSearching: (isSearching) => set({ isSearching }),
  setFilters: (filters) => set({ filters }),
  setFilterOptions: (filterOptions) => set({ filterOptions }),
  setUnavailable: (unavailable) => set({ unavailable }),
  setSearchError: (searchError) => set({ searchError }),
  setSelectedFile: (selectedFile) => set({ selectedFile }),
  setActiveIndex: (activeIndex) => set({ activeIndex }),
}))
