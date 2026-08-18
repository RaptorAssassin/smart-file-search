import { Sidebar, SidebarContent, SidebarFooter, SidebarHeader } from '@/components/ui/sidebar'
import DebugMenu from './debug-menu'
import SettingsMenu from './settings-menu'
import IndexingStatus from './indexing-status'
import { FileSearchCorner } from 'lucide-react'

export default function AppSidebar({ className }: { className?: string }) {
  return (
    <Sidebar collapsible="none" className={className}>
      <SidebarHeader>
        <div className="flex items-center justify-center gap-2 whitespace-nowrap select-none">
          <FileSearchCorner className="size-6 shrink-0 text-accent" strokeWidth={2.25} />
          <span className="text-lg font-semibold tracking-tight text-accent">
            Smart File Search
          </span>
        </div>
      </SidebarHeader>
      <SidebarContent></SidebarContent>
      <SidebarFooter>
        <div className="flex w-full flex-col items-center justify-center gap-2">
          <IndexingStatus />
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
