import { Sidebar, SidebarContent, SidebarFooter } from '@/components/ui/sidebar';
import { Button } from '@/components/ui/button';
import DebugMenu from './debug-menu';

export default function AppSidebar() {
  return (
    <Sidebar>
      <SidebarContent></SidebarContent>
      <SidebarFooter>
        <DebugMenu openButton={<Button>Open Debug Menu</Button>} />
      </SidebarFooter>
    </Sidebar>
  );
}
