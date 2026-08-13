import { Theme } from '@/bindings/bindings'
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogTrigger } from './ui/dialog'
import { Field, FieldLabel } from './ui/field'
import {
  Select,
  SelectContent,
  SelectGroup,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from './ui/select'
import { applyTheme } from '@/lib/theme'
import { useConfigStore } from '@/stores/config-store'
import { Checkbox } from './ui/checkbox'
import { useUIStore } from '@/stores/ui-store'
import { Button } from './ui/button'
import { SettingsIcon } from 'lucide-react'
import { Tooltip, TooltipContent, TooltipTrigger } from './ui/tooltip'
import KeyboardShortcut from './keyboard-shortcut'

export default function SettingsMenu() {
  const config = useConfigStore((state) => state.config)
  const setConfig = useConfigStore((state) => state.setConfig)
  const saveConfig = useConfigStore((state) => state.saveConfig)

  const selectedTheme = config?.settings?.theme ?? 'System'

  const handleThemeChange = (value: Theme) => {
    if (!config) return

    setConfig({
      ...config,
      settings: {
        ...(config.settings ?? {}),
        theme: value,
      },
    })

    applyTheme(value)
    void saveConfig()
  }

  const settingsOpen = useUIStore((state) => state.settingsOpen)
  const setSettingsOpen = useUIStore((state) => state.setSettingsOpen)

  return (
    <Dialog open={settingsOpen} onOpenChange={setSettingsOpen}>
      <DialogTrigger
        render={
          <Button type="button">
            <SettingsIcon />
            Settings
            <KeyboardShortcut identifier="settings" />
          </Button>
        }
      />
      <DialogContent>
        <DialogHeader>
          <DialogTitle>Settings</DialogTitle>
        </DialogHeader>
        {/* App Theme */}
        <Field>
          <FieldLabel>App Theme</FieldLabel>
          <Select
            items={[
              { label: 'Light', value: 'Light' },
              { label: 'Dark', value: 'Dark' },
              { label: 'System', value: 'System' },
            ]}
            value={selectedTheme}
            onValueChange={(value) => handleThemeChange(value as Theme)}
          >
            <SelectTrigger>
              <SelectValue placeholder="Select a theme" />
            </SelectTrigger>
            <SelectContent alignItemWithTrigger={false}>
              <SelectGroup>
                <SelectItem value="System">System</SelectItem>
                <SelectItem value="Dark">Dark</SelectItem>
                <SelectItem value="Light">Light</SelectItem>
              </SelectGroup>
            </SelectContent>
          </Select>
        </Field>
        {/* Disable Keyboard Shortcut Hints */}
        <Field orientation="horizontal">
          <Checkbox
            id="disable-keyboard-shortcut-hints"
            name="disable-keyboard-shortcut-hints"
            className="h-4 w-4"
            checked={config?.settings?.disable_keyboard_shortcut_hints ?? false}
            onCheckedChange={(value) => {
              if (!config) return

              const disableKeyboardShortcutHints = value === true

              setConfig({
                ...config,
                settings: {
                  ...(config.settings ?? {}),
                  disable_keyboard_shortcut_hints: disableKeyboardShortcutHints,
                },
              })

              void saveConfig()
            }}
          />
          <FieldLabel>Disable Keyboard Shortcut Hints</FieldLabel>
        </Field>
        {/* Enable Debug Menu */}
        <Field orientation="horizontal">
          <Checkbox
            id="enable-debug-menu"
            name="enable-debug-menu"
            className="h-4 w-4"
            checked={config?.settings?.enable_debug_menu ?? false}
            onCheckedChange={(value) => {
              if (!config) return

              const enableDebugMenu = value === true

              setConfig({
                ...config,
                settings: {
                  ...(config.settings ?? {}),
                  enable_debug_menu: enableDebugMenu,
                },
              })

              void saveConfig()
            }}
          />
          <FieldLabel htmlFor="enable-debug-menu">Enable Debug Menu</FieldLabel>
        </Field>
      </DialogContent>
    </Dialog>
  )
}
