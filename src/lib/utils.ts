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
