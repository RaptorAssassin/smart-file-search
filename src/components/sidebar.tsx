import { Sidebar, SidebarContent, SidebarFooter } from '@/components/ui/sidebar';
import { Button } from '@/components/ui/button';
import DebugMenu from './debug-menu';
import SettingsMenu from './settings-menu';
import { BugIcon, SettingsIcon } from 'lucide-react';

export default function AppSidebar() {
  return (
    <Sidebar collapsible="none">
      <SidebarContent></SidebarContent>
      <SidebarFooter>
        <SettingsMenu
          openButton={
            <Button>
              <SettingsIcon />
              Open Settings
            </Button>
          }
        />
        <DebugMenu
          openButton={
            <Button>
              <BugIcon /> Open Debug Menu
            </Button>
          }
        />
      </SidebarFooter>
    </Sidebar>
  );
}
