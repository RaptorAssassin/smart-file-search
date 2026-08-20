import { invoke } from '@tauri-apps/api/core'
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from '@/components/ui/dialog'
import { useEffect, useState } from 'react'
import { formatBytes } from '@/lib/utils'
import { useConfigStore } from '@/stores/config-store'
import { Field, FieldContent, FieldLabel, FieldLegend, FieldSet } from './ui/field'
import { useUIStore } from '@/stores/ui-store'
import { Button } from './ui/button'
import { BugIcon, InfoIcon } from 'lucide-react'
import KeyboardShortcut from './keyboard-shortcut'
import { Tooltip, TooltipContent, TooltipTrigger } from './ui/tooltip'

type DebugInfo = {
  databasePath: string | null
  databaseSize: number | null
}

export default function DebugMenu() {
  const debugOpen = useUIStore((state) => state.debugOpen)
  const setDebugOpen = useUIStore((state) => state.setDebugOpen)

  const [debugInfo, setDebugInfo] = useState<DebugInfo>({
    databasePath: null,
    databaseSize: null,
  })
  useEffect(() => {
    const fetchDatabaseInfo = async () => {
      try {
        const path: string = await invoke('get_database_path')
        setDebugInfo((prev) => ({ ...prev, databasePath: path }))
        const size: number = await invoke('get_database_size')
        setDebugInfo((prev) => ({ ...prev, databaseSize: size }))
      } catch (error) {
        console.error('Error fetching database info', error)
      }
    }
    fetchDatabaseInfo()
  }, [])

  const config = useConfigStore((state) => state.config)

  if (!config?.settings?.enable_debug_menu) {
    return null
  }

  return (
    <Dialog open={debugOpen} onOpenChange={setDebugOpen}>
      <DialogTrigger
        render={
          <Button type="button">
            <BugIcon />
            Debug
            <KeyboardShortcut identifier="debug" />
          </Button>
        }
      ></DialogTrigger>
      <DialogContent className="scrollbar-hidden flex max-h-[calc(100vh-8rem)] flex-col overflow-y-auto sm:max-w-2xl">
        <DialogHeader>
          <DialogTitle className="text-lg">Debug</DialogTitle>
        </DialogHeader>

        <div className="flex flex-col gap-6">
          <div className="rounded-lg border p-4">
            <FieldSet>
              <FieldLegend>Database</FieldLegend>
              <Field orientation="horizontal">
                <FieldLabel>Database Path</FieldLabel>
                <FieldContent>
                  <code className="break-all font-mono text-xs leading-relaxed text-muted-foreground">
                    {debugInfo.databasePath ?? 'Loading…'}
                  </code>
                </FieldContent>
              </Field>
              <Field orientation="horizontal">
                <FieldLabel>Database Size</FieldLabel>
                <FieldContent className="flex justify-start items-center gap-2 flex-row">
                  <span className="font-mono text-sm">
                    {formatBytes(debugInfo.databaseSize ?? 0)}
                  </span>
                  <Tooltip>
                    <TooltipTrigger render={<InfoIcon className="size-5" />} />
                    <TooltipContent>
                      This only counts the size of the actual .db file, the size may be different
                      after restarting the app when the temporary .db-wal file got merged into the
                      database.
                    </TooltipContent>
                  </Tooltip>
                </FieldContent>
              </Field>
            </FieldSet>
          </div>

          <div className="rounded-lg border p-4">
            <FieldSet>
              <FieldLegend>Configuration</FieldLegend>
              <pre className="scrollbar-hidden max-h-[40vh] overflow-y-auto rounded-md bg-muted/50 p-3 font-mono text-xs leading-relaxed break-all whitespace-pre-wrap">
                {JSON.stringify(config, null, 2)}
              </pre>
            </FieldSet>
          </div>
        </div>
      </DialogContent>
    </Dialog>
  )
}
