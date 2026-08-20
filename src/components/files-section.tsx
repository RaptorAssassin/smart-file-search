import { useCallback, useEffect, useRef, type ReactNode } from 'react'
import { motion, type Variants } from 'framer-motion'
import { commands, type SearchResult } from '@/bindings/bindings'
import { useSearchStore } from '@/stores/search-store'
import { cn, formatBytes, highlightMatches } from '@/lib/utils'
import { modifierKey } from '@/lib/shortcuts'
import { FileSearchIcon, TriangleAlertIcon } from 'lucide-react'
import { FileIcon } from '@/components/file-icon'
import { Skeleton } from '@/components/ui/skeleton'
import { useResultsNavigation } from '@/hooks/use-results-navigation'

const FILE_LIMIT = 100

const listVariants: Variants = {
  hidden: {},
  show: {
    transition: { staggerChildren: 0.06 },
  },
}

const itemVariants: Variants = {
  hidden: { opacity: 0, y: 24 },
  show: { opacity: 1, y: 0, transition: { duration: 0.35, ease: 'easeOut' } },
}

const ENGINE_LABELS: Record<string, string> = {
  vector: 'Semantic search',
  fts: 'Content search',
  metadata: 'Filename search',
}

export default function FilesSection() {
  const searchQuery = useSearchStore((state) => state.searchQuery)
  const files = useSearchStore((state) => state.files)
  const setFiles = useSearchStore((state) => state.setFiles)
  const isSearching = useSearchStore((state) => state.isSearching)
  const setIsSearching = useSearchStore((state) => state.setIsSearching)
  const filters = useSearchStore((state) => state.filters)
  const filterOptions = useSearchStore((state) => state.filterOptions)
  const unavailable = useSearchStore((state) => state.unavailable)
  const setUnavailable = useSearchStore((state) => state.setUnavailable)
  const searchError = useSearchStore((state) => state.searchError)
  const setSearchError = useSearchStore((state) => state.setSearchError)
  const activeIndex = useSearchStore((state) => state.activeIndex)
  const setActiveIndex = useSearchStore((state) => state.setActiveIndex)
  const setSelectedFile = useSearchStore((state) => state.setSelectedFile)

  const scrollRef = useRef<HTMLDivElement>(null)

  useResultsNavigation(scrollRef)

  const searchFiles = useCallback(
    async (query: string) => {
      try {
        const result = await commands.searchFiles(query, filters, FILE_LIMIT)
        if (result.status === 'ok') {
          setUnavailable(result.data.unavailable)
          setSearchError(null)
          return result.data.results
        }
        setSearchError(result.error)
        setUnavailable([])
        return []
      } catch (error) {
        setSearchError(String(error))
        setUnavailable([])
        return []
      }
    },
    [filters, setUnavailable, setSearchError]
  )

  useEffect(() => {
    const trimmed = searchQuery?.trim() ?? ''
    if (!trimmed) {
      setFiles([])
      setIsSearching(false)
      setUnavailable([])
      setSearchError(null)
      setActiveIndex(null)
      return
    }

    let cancelled = false
    setIsSearching(true)
    const timeout = setTimeout(() => {
      void searchFiles(trimmed).then((results) => {
        if (!cancelled) {
          setFiles(results)
          setIsSearching(false)
          setActiveIndex(results.length > 0 ? 0 : null)
        }
      })
    }, 300)

    return () => {
      cancelled = true
      clearTimeout(timeout)
    }
  }, [
    searchQuery,
    searchFiles,
    setFiles,
    setIsSearching,
    setUnavailable,
    setSearchError,
    setActiveIndex,
  ])

  const isEmpty = !searchQuery || searchQuery.trim() === ''
  const showSkeletons = isSearching && files.length === 0
  const stillIndexing = (filterOptions?.extensions.length ?? 0) === 0
  const unavailableNames = unavailable.map((name) => ENGINE_LABELS[name] ?? name).filter(Boolean)

  return (
    <div ref={scrollRef} className="scrollbar-hidden h-full overflow-y-auto">
      {showSkeletons ? (
        <div className="flex flex-col gap-2 p-4">
          {Array.from({ length: 6 }).map((_, i) => (
            <Skeleton key={i} className="h-16 w-full rounded-lg" />
          ))}
        </div>
      ) : isEmpty ? (
        <EmptyState>
          <FileSearchIcon className="size-8 text-muted-foreground" />
          {stillIndexing ? (
            <>
              <span className="text-base font-medium">Indexing your files…</span>
              <span>Results will appear once the first scan finishes.</span>
            </>
          ) : (
            <>
              <span className="text-base font-medium">Search everything on your device</span>
              <span>Press {modifierKey()} + K to focus the search bar.</span>
            </>
          )}
        </EmptyState>
      ) : searchError ? (
        <EmptyState>
          <TriangleAlertIcon className="size-8 text-destructive" />
          <span className="text-base font-medium">Search failed</span>
          <span>{searchError}</span>
        </EmptyState>
      ) : files.length === 0 ? (
        <EmptyState>
          {unavailableNames.length > 0 ? (
            <>
              <TriangleAlertIcon className="size-8 text-muted-foreground" />
              <span>No results — {unavailableNames.join(', ')} unavailable.</span>
            </>
          ) : (
            <>
              <FileSearchIcon className="size-8 text-muted-foreground" />
              <span>No results for “{searchQuery?.trim()}”</span>
            </>
          )}
        </EmptyState>
      ) : (
        <>
          {unavailableNames.length > 0 && (
            <div className="flex items-center gap-2 px-4 pt-3 text-xs text-muted-foreground">
              <TriangleAlertIcon className="size-4 shrink-0" />
              {unavailableNames.join(', ')} unavailable — showing remaining results.
            </div>
          )}
          <div className="px-4 pt-3 text-xs text-muted-foreground">
            {files.length} {files.length === 1 ? 'result' : 'results'}
          </div>
          <motion.ul
            key={searchQuery}
            variants={listVariants}
            initial="hidden"
            whileInView="show"
            viewport={{ root: scrollRef, once: true }}
            className="flex flex-col gap-2 p-4"
          >
            {files.map((file, index) => (
              <motion.li key={file.file_id} variants={itemVariants} data-index={index}>
                <FileCard
                  file={file}
                  query={searchQuery ?? ''}
                  active={activeIndex === index}
                  onSelect={() => setSelectedFile(file)}
                  onFocus={() => setActiveIndex(index)}
                />
              </motion.li>
            ))}
          </motion.ul>
        </>
      )}
    </div>
  )
}

function EmptyState({ children }: { children: ReactNode }) {
  return (
    <div className="flex h-full flex-col items-center justify-center gap-2 p-4 text-center text-sm text-muted-foreground">
      {children}
    </div>
  )
}

function FileCard({
  file,
  query,
  active,
  onSelect,
  onFocus,
}: {
  file: SearchResult
  query: string
  active: boolean
  onSelect: () => void
  onFocus: () => void
}) {
  return (
    <button
      type="button"
      onClick={onSelect}
      onFocus={onFocus}
      className={cn(
        'flex w-full items-center gap-3 rounded-lg border border-border bg-background p-3 text-left transition-colors hover:bg-muted',
        active && 'border-accent bg-accent/10 hover:bg-accent/10'
      )}
    >
      <FileIcon extension={file.extension} />
      <span className="flex min-w-0 flex-1 flex-col">
        <span className="truncate text-sm font-medium">
          {highlightMatches(file.file_name, query)}
        </span>
        <span className="truncate text-xs text-muted-foreground">
          {highlightMatches(file.file_path, query)}
        </span>
      </span>
      <span className="flex shrink-0 flex-col items-end gap-1">
        <span className="text-xs text-muted-foreground">{formatBytes(file.file_size)}</span>
        <span className="text-xs text-muted-foreground">
          {file.modified_at?.slice(0, 10) ?? ''}
        </span>
      </span>
    </button>
  )
}
