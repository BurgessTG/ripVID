/**
 * Shared TypeScript interfaces and types for ripVID Desktop App
 */

export interface DownloadProgress {
  percent: number
  speed: string
  eta: string
}

export type DownloadFormat = 'mp3' | 'mp4'

export interface ArchiveItem {
  id: string
  title: string
  url: string
  platform: string
  date: string
  path: string
  format: DownloadFormat
  /** Whether the file was still on disk the last time it was checked */
  fileExists?: boolean
}

export interface DownloadStarted {
  id: string
  path: string
}

/** A file discovered by the `scan_downloads_folder` command */
export interface ScannedFile {
  path: string
  filename: string
  format: DownloadFormat
  size: number
  modified?: number
}

export type DownloadStatus =
  | 'idle'
  | 'downloading'
  | 'processing'
  | 'success'
  | 'error'
  | 'cancelled'

export type VideoQuality = 'best' | '2160p' | '1440p' | '1080p' | '720p' | '480p' | '360p'

export type BrowserType = 'firefox' | 'chrome' | 'edge' | 'brave' | 'safari' | 'opera'

export interface QualityOption {
  value: VideoQuality
  label: string
  description: string
}

export interface BrowserOption {
  value: BrowserType
  label: string
  available: boolean
}

export interface DownloadSettings {
  quality: VideoQuality
  useCookies: boolean
  browser: BrowserType | null
}
