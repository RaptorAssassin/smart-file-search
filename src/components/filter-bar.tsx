import { useEffect, useMemo, useState } from 'react'
import { SlidersHorizontalIcon, XIcon } from 'lucide-react'
import { commands } from '@/bindings/bindings'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { useSearchStore } from '@/stores/search-store'
import { cn } from '@/lib/utils'

const VISIBLE_CHIPS = 15

function parseMb(value: string): number | null {
  if (value.trim() === '') return null
  const mb = Number(value)
  if (Number.isNaN(mb) || mb < 0) return null
  return Math.round(mb * 1024 * 1024)
}

function toMb(bytes: number | null): string {
  if (bytes === null || bytes === 0) return ''
  return String(Number((bytes / (1024 * 1024)).toFixed(2)))
}

export default function FilterBar() {
  const filterOptions = useSearchStore((state) => state.filterOptions)
  const setFilterOptions = useSearchStore((state) => state.setFilterOptions)
  const filters = useSearchStore((state) => state.filters)
  const setFilters = useSearchStore((state) => state.setFilters)

  const [showSizeDate, setShowSizeDate] = useState(false)
  const [showAllExtensions, setShowAllExtensions] = useState(false)

  useEffect(() => {
    void commands.searchFilterOptions().then((result) => {
      if (result.status === 'ok') setFilterOptions(result.data)
    })
  }, [setFilterOptions])

  const extensions = filterOptions?.extensions ?? []
  const visibleExtensions = showAllExtensions ? extensions : extensions.slice(0, VISIBLE_CHIPS)

  const hasFilters = useMemo(
    () =>
      filters.extensions.length > 0 ||
      filters.min_size !== null ||
      filters.max_size !== null ||
      filters.modified_after !== null ||
      filters.modified_before !== null,
    [filters]
  )

  const toggleExtension = (extension: string) => {
    const selected = filters.extensions.includes(extension)
    setFilters({
      ...filters,
      extensions: selected
        ? filters.extensions.filter((e) => e !== extension)
        : [...filters.extensions, extension],
    })
  }

  const clearFilters = () => {
    setFilters({
      extensions: [],
      min_size: null,
      max_size: null,
      modified_after: null,
      modified_before: null,
      created_after: null,
      created_before: null,
    })
    setShowSizeDate(false)
    setShowAllExtensions(false)
  }

  if (extensions.length === 0 && !hasFilters) return null

  return (
    <div className="flex flex-col gap-2 border-b border-border px-4 py-2">
      <div className="flex flex-wrap items-center gap-1.5">
        {visibleExtensions.map((extension) => {
          const active = filters.extensions.includes(extension)
          return (
            <Button
              key={extension}
              type="button"
              size="sm"
              variant={active ? 'default' : 'outline'}
              className={cn(active && 'h-6 px-2 text-xs')}
              onClick={() => toggleExtension(extension)}
            >
              {extension}
            </Button>
          )
        })}
        {extensions.length > VISIBLE_CHIPS && !showAllExtensions && (
          <Button
            type="button"
            size="sm"
            variant="ghost"
            className="h-6 px-2 text-xs text-muted-foreground"
            onClick={() => setShowAllExtensions(true)}
          >
            +{extensions.length - VISIBLE_CHIPS} more
          </Button>
        )}
        <Button
          type="button"
          size="sm"
          variant="ghost"
          className={cn(
            'h-6 px-2 text-xs text-muted-foreground',
            showSizeDate && 'text-foreground'
          )}
          onClick={() => setShowSizeDate((v) => !v)}
        >
          <SlidersHorizontalIcon />
          Size &amp; date
        </Button>
        {hasFilters && (
          <Button
            type="button"
            size="sm"
            variant="ghost"
            className="h-6 px-2 text-xs text-destructive"
            onClick={clearFilters}
          >
            <XIcon />
            Clear
          </Button>
        )}
      </div>

      {showSizeDate && (
        <div className="grid grid-cols-2 gap-2 sm:grid-cols-4">
          <label className="flex flex-col gap-1 text-xs text-muted-foreground">
            Min size (MB)
            <Input
              type="number"
              min="0"
              step="0.01"
              value={toMb(filters.min_size)}
              placeholder="0"
              onChange={(e) => setFilters({ ...filters, min_size: parseMb(e.target.value) })}
            />
          </label>
          <label className="flex flex-col gap-1 text-xs text-muted-foreground">
            Max size (MB)
            <Input
              type="number"
              min="0"
              step="0.01"
              value={toMb(filters.max_size)}
              placeholder="∞"
              onChange={(e) => setFilters({ ...filters, max_size: parseMb(e.target.value) })}
            />
          </label>
          <label className="flex flex-col gap-1 text-xs text-muted-foreground">
            Modified after
            <Input
              type="date"
              value={filters.modified_after ?? ''}
              onChange={(e) => setFilters({ ...filters, modified_after: e.target.value || null })}
            />
          </label>
          <label className="flex flex-col gap-1 text-xs text-muted-foreground">
            Modified before
            <Input
              type="date"
              value={filters.modified_before ?? ''}
              onChange={(e) => setFilters({ ...filters, modified_before: e.target.value || null })}
            />
          </label>
        </div>
      )}
    </div>
  )
}
