import { invoke } from '@tauri-apps/api/core'
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from '@/components/ui/dialog'
import { useEffect, useState } from 'react'
import { copyToClipboard, formatBytes } from '@/lib/utils'
import { useConfigStore } from '@/stores/config-store'
import { Field, FieldContent, FieldLabel } from './ui/field'
import { useUIStore } from '@/stores/ui-store'
import { Button } from './ui/button'
import { BugIcon, CopyIcon } from 'lucide-react'
import KeyboardShortcut from './keyboard-shortcut'

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
      <DialogContent className="min-w-1/2 min-h-1/2 flex flex-col">
        <DialogHeader>
          <DialogTitle>Debug</DialogTitle>
        </DialogHeader>

        <Field>
          <FieldLabel>Database Path</FieldLabel>
          {debugInfo.databasePath}
        </Field>
        <Field>
          <FieldLabel>Database Size</FieldLabel>
          {formatBytes(debugInfo.databaseSize ?? 0)}
        </Field>

        <Field>
          <FieldLabel className="flex gap-2">
            Config{' '}
            <Button onClick={() => copyToClipboard(JSON.stringify(config, null, 2))} size="icon-xs">
              <CopyIcon />
            </Button>
          </FieldLabel>
          <pre className="border-border border-2 p-2 rounded-(--radius)">
            <code>{JSON.stringify(config, null, 2)}</code>
          </pre>
        </Field>
      </DialogContent>
    </Dialog>
  )
}
