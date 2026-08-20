import { useEffect, useState } from 'react'
import { commands, type UsageSnapshot } from '@/bindings/bindings'

export function useUsage(intervalMs = 2000): UsageSnapshot | null {
  const [usage, setUsage] = useState<UsageSnapshot | null>(null)

  useEffect(() => {
    let cancelled = false

    const poll = async () => {
      const snapshot = await commands.getUsage()
      if (!cancelled) setUsage(snapshot)
    }

    void poll()
    const id = setInterval(() => void poll(), intervalMs)

    return () => {
      cancelled = true
      clearInterval(id)
    }
  }, [intervalMs])

  return usage
}
