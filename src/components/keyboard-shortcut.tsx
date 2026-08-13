import { useConfigStore } from '@/stores/config-store'
import { Kbd, KbdGroup } from './ui/kbd'
import { KEYBOARD_SHORTCUTS, SHORTCUTS } from '@/lib/shortcuts'

type ShortcutIdentifier = (typeof SHORTCUTS)[number]['identifier']

type KeyboardShortcutProps = {
  identifier: ShortcutIdentifier
}

export default function KeyboardShortcut({ identifier }: KeyboardShortcutProps) {
  // Don't show keyboard shortcut hints if the user has disabled them in the config
  const config = useConfigStore((state) => state.config)
  if (config?.settings?.disable_keyboard_shortcut_hints) return null


  const shortcut = KEYBOARD_SHORTCUTS[identifier]
  if (!shortcut) return null

  return (
    <KbdGroup>
      {shortcut.map((label) => (
        <Kbd key={label}>{label}</Kbd>
      ))}
    </KbdGroup>
  )
}
