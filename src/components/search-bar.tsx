import { useEffect, useRef } from 'react'
import { Button } from './ui/button'
import { SearchIcon, XIcon } from 'lucide-react'
import { InputGroup, InputGroupAddon, InputGroupInput } from './ui/input-group'
import { useUIStore } from '@/stores/ui-store'
import { useSearchStore } from '@/stores/search-store'
import { isMacOS } from '@/lib/platform'
import KeyboardShortcut from './keyboard-shortcut'

export default function SearchBar() {
  const searchQuery = useSearchStore((state) => state.searchQuery) ?? ''
  const setSearchQuery = useSearchStore((state) => state.setSearchQuery)
  const inputRef = useRef<HTMLInputElement>(null)

  const setSearchBarFocused = useUIStore((state) => state.setSearchBarFocused)

  const handleSearchQueryChange = (query: string) => {
    setSearchQuery(query)
    if (query.trim() === '') return
  }

  const isMac = isMacOS()

  // Focus search bar when Ctrl/Cmd+K is pressed and blur on Escape
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      const modifierPressed = isMac ? e.metaKey : e.ctrlKey

      if (modifierPressed && e.key.toLowerCase() === 'k') {
        e.preventDefault()

        inputRef.current?.focus()
        inputRef.current?.select()
        return
      }

      if (e.key === 'Escape' || e.key === 'Esc') {
        if (document.activeElement === inputRef.current) {
          inputRef.current?.blur()
        }
      }
    }

    window.addEventListener('keydown', handleKeyDown)

    inputRef.current?.focus()
    setSearchBarFocused(true)

    return () => {
      window.removeEventListener('keydown', handleKeyDown)
    }
  }, [isMac, setSearchBarFocused])

  return (
    <div className="p-4">
      <div className="rounded-(--radius) relative flex items-center justify-center">
        <InputGroup className="max-w-lg">
          <InputGroupInput
            type="text"
            placeholder="Search for a file..."
            className="h-full w-full border-none bg-transparent p-2 focus:border-none focus:outline-none transition-shadow transition-duration-100"
            value={searchQuery}
            onChange={(e) => handleSearchQueryChange(e.target.value)}
            onFocus={() => setSearchBarFocused(true)}
            onBlur={() => setSearchBarFocused(false)}

            ref={inputRef}
            autoComplete="off"
            inputMode="search"
            spellCheck={false}
          />
          <InputGroupAddon>
            <SearchIcon />
          </InputGroupAddon>
          <InputGroupAddon align="inline-end" className="relative">
            {searchQuery ? (
              <div className="absolute right-1 top-1/2 -translate-y-1/2">
                <Button
                  variant="secondary"
                  size="icon-xs"
                  className=" active:translate-y-0 data-[slot=button]:active:scale-95 "
                  onClick={() => setSearchQuery('')}
                >
                  <XIcon />
                </Button>
              </div>
            ) : (
              <KeyboardShortcut identifier="search" />
            )}
          </InputGroupAddon>
        </InputGroup>
      </div>
    </div>
  )
}
