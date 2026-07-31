import { invoke } from '@tauri-apps/api/core';
import './App.css';
import { useEffect, useState } from 'react';
import { Input } from './components/ui/input';
import { SidebarProvider } from './components/ui/sidebar';
import AppSidebar from './components/sidebar';

function App() {
  useEffect(() => {
    const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)');

    const handleThemeChange = (event: MediaQueryListEvent | MediaQueryList) => {
      const root = document.getElementById('root');
      if (!root) return;
      if (event.matches) {
        root.classList.add('dark');
      } else {
        root.classList.remove('dark');
      }

      handleThemeChange(mediaQuery);

      mediaQuery.addEventListener('change', handleThemeChange);

      return () => {
        mediaQuery.removeEventListener('change', handleThemeChange);
      };
    };
  }, []);

  const [searchQuery, setSearchQuery] = useState('');

  return (
    <div className="bg-background text-foreground w-screen h-screen relative">
      <SidebarProvider>
        <div className="w-full h-full flex">
          <AppSidebar />
          <div className="w-full h-full relative">
            <div className="absolute top-3 left-1/2 transform -translate-x-1/2">
              <div className="rounded-2xl bg-accent min-w-80 h-8 flex items-center justify-center p-2">
                <Input
                  type="text"
                  placeholder="Search for a file..."
                  className="bg-transparent border-none focus:border-none focus-visible:ring-0 focus:outline-none  w-full h-full"
                  value={searchQuery}
                  onChange={(e) => setSearchQuery(e.target.value)}
                />
              </div>
            </div>
          </div>
        </div>
        <div className="w-full h-full">{searchQuery}</div>
      </SidebarProvider>
    </div>
  );
}

export default App;
