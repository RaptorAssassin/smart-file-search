import { useEffect } from 'react'
import { type Theme } from '@/bindings/bindings'

export const applyTheme = (theme: Theme) => {
  const root = document.documentElement
  const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)')
  switch (theme) {
    case 'Dark':
      root.classList.add('dark')
      break

    case 'Light':
      root.classList.remove('dark')
      break

    case 'System':
      root.classList.toggle('dark', mediaQuery.matches)
      break
  }
}

export function useApplyTheme(theme: Theme | null | undefined) {
  useEffect(() => {
    const resolved =
      theme ?? (window.matchMedia('(prefers-color-scheme: dark)').matches ? 'Dark' : 'Light')
    applyTheme(resolved)

    if (resolved !== 'System') return

    const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)')
    const handleThemeChange = (event: MediaQueryListEvent) => {
      document.documentElement.classList.toggle('dark', event.matches)
    }

    mediaQuery.addEventListener('change', handleThemeChange)
    return () => {
      mediaQuery.removeEventListener('change', handleThemeChange)
    }
  }, [theme])
}
