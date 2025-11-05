/**
 * Application Constants
 * Centralized constants to avoid magic strings and improve maintainability
 */

/**
 * Tauri Event Names
 * Events emitted from Rust backend to frontend
 */
export const EVENTS = {
    DOWNLOAD_PROGRESS: "download-progress",
    DOWNLOAD_STARTED: "download-started",
    DOWNLOAD_STATUS: "download-status",
    DOWNLOAD_PROCESSING: "download-processing",
    DOWNLOAD_COMPLETE: "download-complete",
    DOWNLOAD_CANCELLED: "download-cancelled",
} as const;

/**
 * localStorage Keys
 * Keys used for persisting app state
 */
export const STORAGE_KEYS = {
    ARCHIVE: "ripvid-archive",
    FORMAT: "ripvid-format",
    QUALITY: "ripvid-quality",
    USE_COOKIES: "ripvid-use-cookies",
    TERMS_ACCEPTED: "ripvid-terms-accepted",
} as const;

/**
 * Platform Identifiers
 * Supported video platforms
 */
export const PLATFORMS = {
    YOUTUBE: "youtube",
    X: "x",
    TWITTER: "twitter",
    FACEBOOK: "facebook",
    INSTAGRAM: "instagram",
    TIKTOK: "tiktok",
    UNKNOWN: "unknown",
} as const;

/**
 * Download Statuses
 * Possible states during download lifecycle
 */
export const DOWNLOAD_STATUS = {
    IDLE: "idle",
    DOWNLOADING: "downloading",
    PROCESSING: "processing",
    SUCCESS: "success",
    ERROR: "error",
    CANCELLED: "cancelled",
} as const;

/**
 * Download Formats
 * Supported output formats
 */
export const FORMATS = {
    MP3: "mp3",
    MP4: "mp4",
} as const;

/**
 * Video Quality Options
 * Available quality settings for video downloads
 */
export const QUALITY_OPTIONS = {
    BEST: "best",
    P1080: "1080p",
    P720: "720p",
    P480: "480p",
    P360: "360p",
} as const;

/**
 * Archive Filter Tabs
 * Filter options for archive panel
 */
export const ARCHIVE_TABS = {
    ALL: "all",
    VIDEO: "video",
    AUDIO: "audio",
} as const;

/**
 * Keyboard Shortcuts
 */
export const KEYBOARD = {
    ENTER: "Enter",
    ESCAPE: "Escape",
    TAB: "Tab",
} as const;

/**
 * Default Values
 */
export const DEFAULTS = {
    QUALITY: QUALITY_OPTIONS.BEST,
    FORMAT: FORMATS.MP4,
    USE_COOKIES: false,
    ARCHIVE_TAB: ARCHIVE_TABS.ALL,
} as const;

/**
 * Type exports for type safety
 */
export type EventName = (typeof EVENTS)[keyof typeof EVENTS];
export type StorageKey = (typeof STORAGE_KEYS)[keyof typeof STORAGE_KEYS];
export type Platform = (typeof PLATFORMS)[keyof typeof PLATFORMS];
export type DownloadStatus = (typeof DOWNLOAD_STATUS)[keyof typeof DOWNLOAD_STATUS];
export type Format = (typeof FORMATS)[keyof typeof FORMATS];
export type QualityOption = (typeof QUALITY_OPTIONS)[keyof typeof QUALITY_OPTIONS];
export type ArchiveTab = (typeof ARCHIVE_TABS)[keyof typeof ARCHIVE_TABS];
