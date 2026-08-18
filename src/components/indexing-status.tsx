import { useUsage } from '@/hooks/use-usage'
import { formatTokens } from '@/lib/utils'

export default function IndexingStatus() {
  const usage = useUsage()

  if (!usage) return null

  return (
    <div className="flex w-full flex-col gap-0.5 rounded-lg border border-border bg-background px-3 py-2">
      <span className="text-xs text-muted-foreground">Indexing</span>
      <span className="text-sm font-medium tabular-nums">
        {usage.files_indexed.toLocaleString()} files
      </span>
      <span className="text-xs text-muted-foreground tabular-nums">
        {usage.files_ai_indexed.toLocaleString()} AI-enriched · {formatTokens(usage.tokens)} tokens
      </span>
    </div>
  )
}
