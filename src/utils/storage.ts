/**
 * localStorage Utilities with Validation
 * Provides type-safe storage operations with Zod validation
 */

import { z } from "zod";
import { STORAGE_KEYS } from "@constants/index";

/**
 * Archive Item Schema
 * Validates archive items loaded from localStorage
 */
export const ArchiveItemSchema = z.object({
    id: z.string(),
    title: z.string(),
    url: z.string(),
    platform: z.string(),
    date: z.string(),
    path: z.string(),
    format: z.enum(["mp3", "mp4"]),
    fileExists: z.boolean().optional(),
});

export const ArchiveSchema = z.array(ArchiveItemSchema);

export type ArchiveItem = z.infer<typeof ArchiveItemSchema>;

/**
 * Safely parse and validate archive data from localStorage
 * Returns empty array if data is invalid or missing
 */
export function loadArchive(): ArchiveItem[] {
    try {
        const saved = localStorage.getItem(STORAGE_KEYS.ARCHIVE);
        if (!saved) {
            return [];
        }

        const parsed = JSON.parse(saved);
        const result = ArchiveSchema.safeParse(parsed);

        if (!result.success) {
            console.error("Invalid archive data in localStorage:", result.error);
            // Clear corrupted data
            localStorage.removeItem(STORAGE_KEYS.ARCHIVE);
            return [];
        }

        return result.data;
    } catch (error) {
        console.error("Failed to load archive from localStorage:", error);
        localStorage.removeItem(STORAGE_KEYS.ARCHIVE);
        return [];
    }
}

/**
 * Safely save archive to localStorage
 */
export function saveArchive(archive: ArchiveItem[]): void {
    try {
        localStorage.setItem(STORAGE_KEYS.ARCHIVE, JSON.stringify(archive));
    } catch (error) {
        console.error("Failed to save archive to localStorage:", error);
    }
}

/**
 * Safely load string value from localStorage with validation
 */
export function loadString(key: string, allowedValues?: string[]): string | null {
    try {
        const value = localStorage.getItem(key);
        if (!value) {
            return null;
        }

        // If allowed values specified, validate against them
        if (allowedValues && !allowedValues.includes(value)) {
            console.warn(`Invalid value "${value}" for key "${key}"`);
            localStorage.removeItem(key);
            return null;
        }

        return value;
    } catch (error) {
        console.error(`Failed to load ${key} from localStorage:`, error);
        return null;
    }
}

/**
 * Safely save string value to localStorage
 */
export function saveString(key: string, value: string): void {
    try {
        localStorage.setItem(key, value);
    } catch (error) {
        console.error(`Failed to save ${key} to localStorage:`, error);
    }
}

/**
 * Safely load boolean value from localStorage
 */
export function loadBoolean(key: string): boolean {
    try {
        const value = localStorage.getItem(key);
        return value === "true";
    } catch (error) {
        console.error(`Failed to load ${key} from localStorage:`, error);
        return false;
    }
}

/**
 * Safely save boolean value to localStorage
 */
export function saveBoolean(key: string, value: boolean): void {
    try {
        localStorage.setItem(key, value.toString());
    } catch (error) {
        console.error(`Failed to save ${key} to localStorage:`, error);
    }
}
