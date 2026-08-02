import './App.css';
import { useEffect, useState } from 'react';
import { Input } from './components/ui/input';
import { SidebarProvider } from './components/ui/sidebar';
import AppSidebar from './components/sidebar';
import { commands, Theme } from './bindings/bindings';
import { useConfigStore } from './stores/config-stores';
import { XIcon } from 'lucide-react';
import { Button } from './components/ui/button';

function App() {
  const setConfig = useConfigStore((state) => state.setConfig);
  const config = useConfigStore((state) => state.config);

  useEffect(() => {
    const root = document.documentElement;
    const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)');

    const applyTheme = (theme: Theme) => {
      switch (theme) {
        case 'Dark':
          root.classList.add('dark');
          break;

        case 'Light':
          root.classList.remove('dark');
          break;

        case 'System':
          root.classList.toggle('dark', mediaQuery.matches);
          break;
      }
    };

    let theme = config?.settings?.theme;

    if (!theme) {
      theme = mediaQuery.matches ? 'Dark' : 'Light';
    }

    applyTheme(theme);

    if (theme !== 'System') {
      return;
    }

    const handleThemeChange = (event: MediaQueryListEvent) => {
      root.classList.toggle('dark', event.matches);
    };

    mediaQuery.addEventListener('change', handleThemeChange);

    return () => {
      mediaQuery.removeEventListener('change', handleThemeChange);
    };
  }, [config?.settings?.theme]);

  useEffect(() => {
    const fetchConfig = async () => {
      try {
        const nextConfig = await commands.getConfig();
        setConfig(nextConfig);
      } catch (error) {
        console.error('Error fetching config', error);
      }
    };

    fetchConfig();
  }, [setConfig]);

  const [searchQuery, setSearchQuery] = useState('');

  return (
    <div className="bg-background text-foreground flex h-screen w-screen selection:bg-foreground selection:text-background">
      <SidebarProvider className="flex h-full w-full">
        <AppSidebar />
        <main className="relative flex min-w-0 flex-1 flex-col overflow-hidden">
          {/* Search Input */}
          <div className="p-4">
            <div className="rounded-(--radius) p-1 relative flex items-center justify-center">
              <Input
                type="text"
                placeholder="Search for a file..."
                className="h-full w-full border-none bg-transparent p-1.5 focus:border-none focus-visible:ring-0 focus:outline-none "
                value={searchQuery}
                onChange={(e) => setSearchQuery(e.target.value)}
              />
              {searchQuery && (
                <div className="absolute right-1 top-1/2 -translate-y-1/2">
                  <Button
                    variant="secondary"
                    size="icon-sm"
                    className=" active:translate-y-0 data-[slot=button]:active:scale-95 "
                    onClick={() => setSearchQuery('')}
                  >
                    <XIcon />
                  </Button>
                </div>
              )}
            </div>
          </div>
          {/* Search Results */}
          <div className="w-full h-full"></div>
        </main>
      </SidebarProvider>
    </div>
  );
}

export default App;
