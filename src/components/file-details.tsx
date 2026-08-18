import { useState } from 'react'
import { openPath } from '@tauri-apps/plugin-opener'
import { CheckIcon, CopyIcon, ExternalLinkIcon, FolderOpenIcon } from 'lucide-react'
import {
  Sheet,
  SheetContent,
  SheetDescription,
  SheetFooter,
  SheetHeader,
  SheetTitle,
} from '@/components/ui/sheet'
import { Button } from '@/components/ui/button'
import { commands } from '@/bindings/bindings'
import { useSearchStore } from '@/stores/search-store'
import { FileIcon } from '@/components/file-icon'
import { copyToClipboard, formatBytes } from '@/lib/utils'

function DetailRow({ label, value }: { label: string; value: string }) {
  return (
    <div className="flex flex-col gap-0.5">
      <span className="text-xs text-muted-foreground">{label}</span>
      <span className="text-sm break-words">{value}</span>
    </div>
  )
}

export default function FileDetails() {
  const selectedFile = useSearchStore((s) => s.selectedFile)
  const setSelectedFile = useSearchStore((s) => s.setSelectedFile)
  const [copied, setCopied] = useState(false)

  const handleOpen = () => {
    if (!selectedFile) return
    void openPath(selectedFile.file_path).catch((e) => console.error(e))
  }

  const handleReveal = () => {
    if (!selectedFile) return
    void commands.revealInFolder(selectedFile.file_path)
  }

  const handleCopy = async () => {
    if (!selectedFile) return
    if (await copyToClipboard(selectedFile.file_path)) {
      setCopied(true)
      setTimeout(() => setCopied(false), 1500)
    }
  }

  return (
    <Sheet
      open={selectedFile !== null}
      onOpenChange={(open) => {
        if (!open) setSelectedFile(null)
      }}
    >
      <SheetContent side="right" className="w-full sm:max-w-md">
        <SheetHeader>
          <div className="flex items-center gap-3">
            <FileIcon extension={selectedFile?.extension ?? ''} />
            <div className="min-w-0">
              <SheetTitle className="truncate">{selectedFile?.file_name}</SheetTitle>
              <SheetDescription className="truncate">{selectedFile?.file_path}</SheetDescription>
            </div>
          </div>
        </SheetHeader>

        <div className="flex flex-col gap-3 px-4">
          <DetailRow label="Size" value={selectedFile ? formatBytes(selectedFile.file_size) : ''} />
          <DetailRow
            label="Modified"
            value={
              selectedFile?.modified_at ? new Date(selectedFile.modified_at).toLocaleString() : '—'
            }
          />
          <DetailRow
            label="Created"
            value={
              selectedFile?.created_at ? new Date(selectedFile.created_at).toLocaleString() : '—'
            }
          />
          <DetailRow
            label="Type"
            value={selectedFile?.mime_type ?? selectedFile?.extension ?? '—'}
          />
          <DetailRow label="Category" value={selectedFile?.category ?? '—'} />
          {selectedFile?.score !== null && selectedFile?.score !== undefined && (
            <DetailRow label="Relevance score" value={selectedFile.score.toFixed(4)} />
          )}
        </div>

        <SheetFooter>
          <div className="flex flex-wrap gap-2">
            <Button onClick={handleOpen}>
              <ExternalLinkIcon />
              Open
            </Button>
            <Button variant="outline" onClick={handleReveal}>
              <FolderOpenIcon />
              Reveal
            </Button>
            <Button variant="outline" onClick={handleCopy}>
              {copied ? <CheckIcon /> : <CopyIcon />}
              {copied ? 'Copied' : 'Copy path'}
            </Button>
          </div>
        </SheetFooter>
      </SheetContent>
    </Sheet>
  )
}
