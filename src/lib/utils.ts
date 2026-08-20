import { createElement, type ReactNode } from 'react'
import { writeText as tauriWriteText } from '@tauri-apps/plugin-clipboard-manager'
import { clsx, type ClassValue } from 'clsx'
import { twMerge } from 'tailwind-merge'

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs))
}

export function formatBytes(bytes: number, decimals = 2): string {
  if (bytes === 0) return '0 Bytes'

  const k = 1024
  const sizes = ['Bytes', 'KB', 'MB', 'GB', 'TB', 'PB']

  const index = Math.floor(Math.log(bytes) / Math.log(k))
  const value = bytes / Math.pow(k, index)

  return `${parseFloat(value.toFixed(decimals))} ${sizes[index]}`
}

export function formatTokens(tokens: number): string {
  if (tokens < 1000) return String(tokens)
  if (tokens < 1_000_000) return `${(tokens / 1000).toFixed(1)}K`
  return `${(tokens / 1_000_000).toFixed(1)}M`
}

const TERM_SPLIT = /\s+/

export function highlightMatches(text: string, query: string): ReactNode[] {
  const terms = query.trim().split(TERM_SPLIT).filter(Boolean)
  if (terms.length === 0) return [text]

  const escaped = terms.map((t) => t.replace(/[.*+?^${}()|[\]\\]/g, '\\$&'))
  const pattern = new RegExp(`(${escaped.join('|')})`, 'gi')
  const lowerTerms = terms.map((t) => t.toLowerCase())

  return text.split(pattern).map((part, i) => {
    if (lowerTerms.includes(part.toLowerCase())) {
      return createElement(
        'mark',
        { key: i, className: 'rounded-sm bg-accent/30 px-0.5 text-foreground' },
        part
      )
    }
    return part
  })
}

export async function copyToClipboard(text: string): Promise<boolean> {
  const isTauri = '__TAURI_INTERNALS__' in window
  if (isTauri) {
    try {
      await tauriWriteText(text)
      return true
    } catch (err) {
      console.error('Failed to copy text via clipboard-manager:', err)
    }
  }
  if (navigator?.clipboard?.writeText) {
    try {
      await navigator.clipboard.writeText(text)
      return true
    } catch (err) {
      console.error('Failed to copy text via Clipboard API:', err)
    }
  }
  try {
    const textarea = document.createElement('textarea')
    textarea.value = text
    textarea.style.position = 'fixed'
    textarea.style.opacity = '0'
    document.body.appendChild(textarea)
    textarea.focus()
    textarea.select()
    const copied = document.execCommand('copy')
    textarea.remove()
    if (copied) return true
  } catch (err) {
    console.error('Failed to copy text via execCommand:', err)
  }
  return false
}
