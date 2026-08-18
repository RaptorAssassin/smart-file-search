import './App.css'
import { useEffect } from 'react'
import { SidebarProvider } from './components/ui/sidebar'
import AppSidebar from './components/sidebar'
import { useConfigStore } from './stores/config-store'
import { useApplyTheme } from './lib/theme'
import SearchBar from './components/search-bar'
import { TooltipProvider } from './components/ui/tooltip'
import { useKeyboardShortcuts } from './lib/shortcuts'
import FilesSection from './components/files-section'
import FilterBar from './components/filter-bar'
import FileDetails from './components/file-details'

function App() {
  const config = useConfigStore((state) => state.config)

  useApplyTheme(config?.settings?.theme)

  useEffect(() => {
    useConfigStore.getState().loadConfig()
  }, [])

  useKeyboardShortcuts()

  return (
    <div className="bg-background text-foreground flex h-screen w-screen selection:bg-foreground selection:text-background">
      <TooltipProvider>
        <SidebarProvider className="flex h-full w-full">
          <AppSidebar className="flex-1 min-w-50 max-w-80" />
          <main className="relative flex min-w-0 flex-3 flex-col overflow-hidden">
            <SearchBar />
            <FilterBar />
            <FilesSection />
          </main>
          <FileDetails />
        </SidebarProvider>
      </TooltipProvider>
    </div>
  )
}

export default App
