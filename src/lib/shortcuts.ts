import { useEffect } from 'react'
import { useUIStore } from '@/stores/ui-store'

const isMac = navigator.platform.toUpperCase().indexOf('MAC') >= 0

export const modifierKey = () => {
  if (isMac) {
    return '⌘'
  }
  return 'Ctrl'
}

export const useKeyboardShortcuts = () => {
  useEffect(() => {
    window.addEventListener('keydown', handleKeyDown)
    return () => {
      window.removeEventListener('keydown', handleKeyDown)
    }
  }, [])
}

const matchesModifiers = (event: KeyboardEvent, requiredModifiers: Modifier[]): boolean => {
  const needsPrimary = requiredModifiers.includes('primary')
  const needsShift = requiredModifiers.includes('shift')
  const needsAlt = requiredModifiers.includes('alt')

  const hasPrimary = primaryPressed(event)
  const hasShift = event.shiftKey
  const hasAlt = event.altKey

  return hasPrimary === needsPrimary && hasShift === needsShift && hasAlt === needsAlt
}

export const handleKeyDown = (event: KeyboardEvent) => {
  const target = event.target as HTMLElement | null
  if (
    target &&
    (target.tagName === 'INPUT' || target.tagName === 'TEXTAREA' || target.isContentEditable)
  ) {
    return
  }

  const matchedShortcut = SHORTCUTS.find((shortcut) => {
    const keyMatches = shortcut.key.toLowerCase() === event.key.toLowerCase()
    const modifiersMatch = matchesModifiers(event, shortcut.modifiers)

    return keyMatches && modifiersMatch
  })

  if (matchedShortcut) {
    event.preventDefault()
    matchedShortcut.action()
  }
}

export const primaryPressed = (event: KeyboardEvent) => (isMac ? event.metaKey : event.ctrlKey)

type Modifier = 'primary' | 'shift' | 'alt'

type Shortcut = {
  modifiers: Modifier[]
  key: string
  identifier: string
  action: () => void
}

export const SHORTCUTS: Shortcut[] = [
  {
    modifiers: ['primary'],
    key: 'k',
    identifier: 'search',
    action: () => {
      useUIStore.getState().setSearchBarFocused(true)
    },
  },
  {
    modifiers: [],
    key: 'Escape',
    identifier: 'cancelSearch',
    action: () => {
      useUIStore.getState().setSearchBarFocused(false)
    },
  },
  {
    modifiers: ['primary'],
    key: ',',
    identifier: 'settings',
    action: () => {
      useUIStore.getState().setSettingsOpen(true)
    },
  },
  {
    modifiers: ['primary'],
    key: 'd',
    identifier: 'debug',
    action: () => {
      useUIStore.getState().setDebugOpen(true)
    },
  },
]

const MODIFIER_MAP: Record<Modifier, string> = {
  primary: modifierKey(),
  shift: 'Shift',
  alt: 'Alt',
}

export const KEYBOARD_SHORTCUTS = SHORTCUTS.reduce<Record<string, string[]>>(
  (shortcuts, shortcut) => {
    const { modifiers, key, identifier } = shortcut
    const modifierKeys = modifiers.map((m) => MODIFIER_MAP[m])
    const keyLabel = key.toUpperCase()

    shortcuts[identifier.toLowerCase()] = [...modifierKeys, keyLabel]

    return shortcuts
  },
  {}
)
