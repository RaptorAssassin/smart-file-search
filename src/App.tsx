import './App.css'
import { useEffect } from 'react'
import { SidebarProvider } from './components/ui/sidebar'
import AppSidebar from './components/sidebar'
import { useConfigStore } from './stores/config-store'
import { applyTheme } from './lib/theme'
import SearchBar from './components/search-bar'
import { Button } from './components/ui/button'
import { TooltipProvider } from './components/ui/tooltip'
import { useKeyboardShortcuts } from './lib/shortcuts'

function App() {
  const config = useConfigStore((state) => state.config)

  useEffect(() => {
    const root = document.documentElement
    const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)')

    let theme = config?.settings?.theme

    if (!theme) {
      theme = mediaQuery.matches ? 'Dark' : 'Light'
    }

    applyTheme(theme)

    if (theme !== 'System') {
      return
    }

    const handleThemeChange = (event: MediaQueryListEvent) => {
      root.classList.toggle('dark', event.matches)
    }

    mediaQuery.addEventListener('change', handleThemeChange)

    return () => {
      mediaQuery.removeEventListener('change', handleThemeChange)
    }
  }, [config?.settings?.theme])

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
            <div className="w-full h-full">
              <span className="text-accent">
                Lorem ipsum dolor sit amet, consectetur adipisicing elit. Totam, ullam aspernatur!
                Consectetur est placeat magni eos. Labore, atque optio harum est debitis doloremque
                sequi, sint consectetur, quos ipsam veniam incidunt?
              </span>
              <Button className="bg-accent hover:bg-accent/80">Test Button</Button>
            </div>
          </main>
        </SidebarProvider>
      </TooltipProvider>
    </div>
  )
}

export default App
