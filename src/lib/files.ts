import { invoke } from "@tauri-apps/api/core";
import { homeDir, join } from "@tauri-apps/api/path";
import type { ArchiveItem, DownloadFormat } from "../types";

/** Folders created under the home directory, one per download format */
const FORMAT_FOLDERS: DownloadFormat[] = ["mp4", "mp3"];

/** Root folder holding every download */
export async function downloadsRoot(): Promise<string> {
    return join(await homeDir(), "ripVID");
}

/** Folder a given format is saved into, created if missing */
export async function ensureFormatFolder(
    format: DownloadFormat,
): Promise<string> {
    const dir = await join(await downloadsRoot(), format.toUpperCase());
    await invoke("create_directory", { path: dir });
    return dir;
}

/** Create the whole download folder structure */
export async function ensureDownloadFolders(): Promise<void> {
    await invoke("create_directory", { path: await downloadsRoot() });
    for (const format of FORMAT_FOLDERS) {
        await ensureFormatFolder(format);
    }
}

/** Whether a file is present on disk, retrying while it is still being written */
export async function fileExists(path: string, retries = 1): Promise<boolean> {
    for (let attempt = 0; attempt < retries; attempt++) {
        if (await invoke<boolean>("file_exists", { path })) return true;
        if (attempt < retries - 1) {
            await new Promise((resolve) => setTimeout(resolve, 500));
        }
    }
    return false;
}

/** Re-check every archived file, marking the ones that disappeared */
export async function withFileExistence(
    items: ArchiveItem[],
): Promise<ArchiveItem[]> {
    return Promise.all(
        items.map(async (item) => {
            try {
                return { ...item, fileExists: await fileExists(item.path) };
            } catch (error) {
                console.error("Failed to verify file:", item.path, error);
                return { ...item, fileExists: false };
            }
        }),
    );
}
