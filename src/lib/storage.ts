import type { ArchiveItem, DownloadFormat } from "../types";

/** localStorage keys used by the app */
const KEYS = {
    archive: "ripvid-archive",
    format: "ripvid-format",
    quality: "ripvid-quality",
    termsAccepted: "ripvid-terms-accepted",
} as const;

export function loadArchive(): ArchiveItem[] {
    const saved = localStorage.getItem(KEYS.archive);
    if (!saved) return [];

    try {
        const parsed = JSON.parse(saved);
        return Array.isArray(parsed) ? (parsed as ArchiveItem[]) : [];
    } catch (error) {
        console.error("Failed to parse stored archive:", error);
        return [];
    }
}

export function saveArchive(archive: ArchiveItem[]): ArchiveItem[] {
    localStorage.setItem(KEYS.archive, JSON.stringify(archive));
    return archive;
}

export function loadFormat(): DownloadFormat | null {
    const saved = localStorage.getItem(KEYS.format);
    return saved === "mp3" || saved === "mp4" ? saved : null;
}

export function saveFormat(format: DownloadFormat): void {
    localStorage.setItem(KEYS.format, format);
}

export function loadQuality(): string | null {
    return localStorage.getItem(KEYS.quality);
}

export function saveQuality(quality: string): void {
    localStorage.setItem(KEYS.quality, quality);
}

export function areTermsAccepted(): boolean {
    return Boolean(localStorage.getItem(KEYS.termsAccepted));
}

export function acceptTerms(): void {
    localStorage.setItem(KEYS.termsAccepted, "true");
}
