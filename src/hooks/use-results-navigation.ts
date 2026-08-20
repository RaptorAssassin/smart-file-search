import { useEffect } from 'react'
import { useSearchStore } from '@/stores/search-store'

/**
 * Arrow keys move the active result while the search input is focused;
 * Enter opens the active result's details, Escape clears the selection.
 */
export function useResultsNavigation(scrollRef: React.RefObject<HTMLDivElement | null>) {
  const files = useSearchStore((s) => s.files)
  const activeIndex = useSearchStore((s) => s.activeIndex)
  const setActiveIndex = useSearchStore((s) => s.setActiveIndex)
  const setSelectedFile = useSearchStore((s) => s.setSelectedFile)

  useEffect(() => {
    if (activeIndex === null || activeIndex < 0 || activeIndex >= files.length) return

    const container = scrollRef.current
    const active = container?.querySelector(`[data-index="${activeIndex}"]`)
    active?.scrollIntoView({ block: 'nearest' })
  }, [activeIndex, files.length, scrollRef])

  useEffect(() => {
    const handleKeyDown = (event: KeyboardEvent) => {
      if (files.length === 0) return

      const target = event.target as HTMLElement | null
      const inEditable =
        target !== null &&
        (target.tagName === 'INPUT' || target.tagName === 'TEXTAREA' || target.isContentEditable)

      switch (event.key) {
        case 'ArrowDown':
          event.preventDefault()
          setActiveIndex(activeIndex === null ? 0 : Math.min(activeIndex + 1, files.length - 1))
          break

        case 'ArrowUp':
          event.preventDefault()
          setActiveIndex(activeIndex === null ? files.length - 1 : Math.max(activeIndex - 1, 0))
          break

        case 'Home':
          if (!inEditable) return
          event.preventDefault()
          setActiveIndex(0)
          break

        case 'End':
          if (!inEditable) return
          event.preventDefault()
          setActiveIndex(files.length - 1)
          break

        case 'Enter':
          if (!inEditable) return
          if (activeIndex !== null && files[activeIndex]) {
            event.preventDefault()
            setSelectedFile(files[activeIndex])
          }
          break

        case 'Escape':
          if (activeIndex !== null) {
            event.preventDefault()
            setActiveIndex(null)
          }
          break
      }
    }

    window.addEventListener('keydown', handleKeyDown)
    return () => {
      window.removeEventListener('keydown', handleKeyDown)
    }
  }, [files, activeIndex, setActiveIndex, setSelectedFile])
}
