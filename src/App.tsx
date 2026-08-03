import './App.css';
import { useEffect } from 'react';
import { SidebarProvider } from './components/ui/sidebar';
import AppSidebar from './components/sidebar';
import { useConfigStore } from './stores/config-stores';
import { applyTheme } from './lib/theme';
import SearchBar from './components/search-bar';

function App() {
  const config = useConfigStore((state) => state.config);

  useEffect(() => {
    const root = document.documentElement;
    const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)');

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
    useConfigStore.getState().loadConfig();
  }, []);

  return (
    <div className="bg-background text-foreground flex h-screen w-screen selection:bg-foreground selection:text-background">
      <SidebarProvider className="flex h-full w-full">
        <AppSidebar />
        <main className="relative flex min-w-0 flex-1 flex-col overflow-hidden">
          <SearchBar />
          <div className="w-full h-full" />
        </main>
      </SidebarProvider>
    </div>
  );
}

export default App;
