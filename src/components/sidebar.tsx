import { Sidebar, SidebarContent, SidebarFooter, SidebarHeader } from '@/components/ui/sidebar'
import DebugMenu from './debug-menu'
import SettingsMenu from './settings-menu'
import { FileSearchCorner } from 'lucide-react'

export default function AppSidebar({ className }: { className?: string }) {
  return (
    <Sidebar collapsible="none" className={className}>
      <SidebarHeader>
        <div className="flex gap-2 items-center justify-center text-accent select-none">
          <FileSearchCorner className="text-2xl size-6 shrink-0" />
          <span className="text-2xl font-bold">Smart File Search</span>
        </div>
      </SidebarHeader>
      <SidebarContent></SidebarContent>
      <SidebarFooter>
        <div className="flex w-full flex-col items-center justify-center gap-2">
          <div className="shrink-0">
            <DebugMenu />
          </div>
          <div className="">
            <SettingsMenu />
          </div>
        </div>
      </SidebarFooter>
    </Sidebar>
  )
}
