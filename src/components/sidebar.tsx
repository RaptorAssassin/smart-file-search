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
        <div className="flex w-full items-center justify-center gap-2">
          <div className="shrink-0">
            <DebugMenu
              openButton={
                <Button>
                  <BugIcon />
                  {/* Open Debug Menu */}
                </Button>
              }
            />
          </div>
          <div className="">
            <SettingsMenu
              openButton={
                <Button>
                  <SettingsIcon />
                  Open Settings
                </Button>
              }
            />
          </div>
        </div>
      </SidebarFooter>
    </Sidebar>
  );
}
