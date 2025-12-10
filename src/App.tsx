import { useState, useEffect, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { homeDir, join } from "@tauri-apps/api/path";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import {
    Save,
    X,
    Music,
    Youtube,
    Globe,
    Play,
    Layers,
    XCircle,
    Power,
    AlertCircle,
    RefreshCw,
} from "lucide-react";
import TitleBar from "./components/TitleBar";
import { UpdateChecker } from "./components/UpdateChecker";
import { TermsAcceptance } from "./components/TermsAcceptance";
import ShaderBackground from "./components/ShaderBackground";
import "./components/TermsAcceptance.css";
import "./App.css";

interface DownloadProgress {
    percent: number;
    speed: string;
    eta: string;
}

interface DownloadStarted {
    id: string;
    path: string;
}

interface ArchiveItem {
    id: string;
    title: string;
    url: string;
    platform: string;
    date: string;
    path: string;
    format: "mp3" | "mp4";
    fileExists?: boolean; // Track if file actually exists on disk
}

function App() {
    const [url, setUrl] = useState("");
    const [isDownloading, setIsDownloading] = useState(false);
    const [progress, setProgress] = useState<DownloadProgress | null>(null);
    const [status, setStatus] = useState<
        | "idle"
        | "downloading"
        | "processing"
        | "success"
        | "error"
        | "cancelled"
    >("idle");
    const [platform, setPlatform] = useState<string | null>(null);
    const [archiveOpen, setArchiveOpen] = useState(false);
    const [archive, setArchive] = useState<ArchiveItem[]>([]);
    const [downloadFormat, setDownloadFormat] = useState<"mp3" | "mp4">("mp4");
    const [archiveTab, setArchiveTab] = useState<"all" | "video" | "audio">(
        "all",
    );
    const [showTerms, setShowTerms] = useState(false);
    const [quality, setQuality] = useState<string>("best");
    const [currentDownloadId, setCurrentDownloadId] = useState<string | null>(
        null,
    );
    const [showSettings, setShowSettings] = useState(false);
    const [processingMessage, setProcessingMessage] = useState<string>("Processing...");
    const [processingStartTime, setProcessingStartTime] = useState<number | null>(null);
    const [processingElapsed, setProcessingElapsed] = useState<number>(0);
    const inputRef = useRef<HTMLInputElement>(null);
    const archivePanelRef = useRef<HTMLDivElement>(null);
    const settingsPanelRef = useRef<HTMLDivElement>(null);
    const downloadInfoRef = useRef<{
        url: string;
        platform: string;
        format: "mp3" | "mp4";
    } | null>(null);
    const lastProgressUpdate = useRef<number>(0);

    useEffect(() => {
        // Listen for download progress (debounced to 100ms to prevent excessive re-renders)
        const progressUnsubscribe = listen<DownloadProgress>(
            "download-progress",
            (event) => {
                const now = Date.now();
                // Only update progress every 100ms to avoid performance issues
                if (now - lastProgressUpdate.current >= 100) {
                    lastProgressUpdate.current = now;
                    setProgress(event.payload);
                    setStatus("downloading");
                }
            },
        );

        // Listen for download started
        const startedUnsubscribe = listen<DownloadStarted>(
            "download-started",
            (event) => {
                console.log("Download started:", event.payload);
                setCurrentDownloadId(event.payload.id);
                setStatus("downloading");
            },
        );

        // Listen for download status messages (from stderr)
        const statusUnsubscribe = listen<string>("download-status", (event) => {
            console.log("Status message:", event.payload);
        });

        // Listen for download processing (ffmpeg merge or audio extraction)
        const processingUnsubscribe = listen<{
            message: string;
            id: string;
        }>("download-processing", (event) => {
            console.log("Processing:", event.payload);
            setStatus("processing");
            setProgress(null); // Clear percentage since we're in processing phase
            // Store the message for display
            setProcessingMessage(event.payload.message);
            // Start the processing timer
            setProcessingStartTime(Date.now());
        });

        // Listen for download completion
        const completeUnsubscribe = listen<{
            success: boolean;
            id: string;
            path?: string;
            error?: string;
        }>("download-complete", async (event) => {
            console.log("Download complete:", event.payload);

            if (
                event.payload.success &&
                event.payload.path &&
                downloadInfoRef.current
            ) {
                // Verify file actually exists before adding to archive
                // Add small delay to ensure file is fully written
                const verifyFileExists = async (retries = 3): Promise<boolean> => {
                    for (let i = 0; i < retries; i++) {
                        const exists = await invoke<boolean>("file_exists", {
                            path: event.payload.path,
                        });
                        if (exists) return true;
                        // Wait 500ms before retry
                        await new Promise(r => setTimeout(r, 500));
                    }
                    return false;
                };

                try {
                    const exists = await verifyFileExists();

                    if (exists) {
                        // Add to archive with fileExists flag
                        const newItem: ArchiveItem = {
                            id: Date.now().toString(),
                            title:
                                downloadInfoRef.current.url.split("/").pop() ||
                                "Download",
                            url: downloadInfoRef.current.url,
                            platform: downloadInfoRef.current.platform,
                            date: new Date().toLocaleDateString(),
                            path: event.payload.path,
                            format: downloadInfoRef.current.format,
                            fileExists: true,
                        };

                        setArchive(prevArchive => {
                            const newArchive = [newItem, ...prevArchive];
                            localStorage.setItem(
                                "ripvid-archive",
                                JSON.stringify(newArchive),
                            );
                            console.log("Added to archive:", newItem);
                            return newArchive;
                        });
                    } else {
                        console.warn(
                            "File not found after download:",
                            event.payload.path,
                        );
                    }
                } catch (error) {
                    console.error("Failed to verify file:", error);
                }
            }

            setStatus(event.payload.success ? "success" : "error");
            setIsDownloading(false);
            setCurrentDownloadId(null);
            downloadInfoRef.current = null;
        });

        // Listen for download cancellation
        const cancelledUnsubscribe = listen<{ id: string; path: string }>(
            "download-cancelled",
            (event) => {
                console.log("Download cancelled:", event.payload);
                setStatus("cancelled");
                setIsDownloading(false);
                setCurrentDownloadId(null);
                downloadInfoRef.current = null;
            },
        );

        return () => {
            progressUnsubscribe.then((fn) => fn());
            startedUnsubscribe.then((fn) => fn());
            statusUnsubscribe.then((fn) => fn());
            processingUnsubscribe.then((fn) => fn());
            completeUnsubscribe.then((fn) => fn());
            cancelledUnsubscribe.then((fn) => fn());
        };
        // eslint-disable-next-line react-hooks/exhaustive-deps
    }, []); // Empty deps - listeners should only be set up once on mount

    useEffect(() => {
        // Initialize app and check first launch
        const initializeApp = async () => {
            // Check if terms have been accepted
            const termsAccepted = localStorage.getItem("ripvid-terms-accepted");
            if (!termsAccepted) {
                setShowTerms(true);
            } else {
                // Ensure folder structure exists
                await setupFolderStructure();
            }

            // Show window once app is ready
            const appWindow = getCurrentWebviewWindow();
            await appWindow.show();

            // Load archive from localStorage
            const saved = localStorage.getItem("ripvid-archive");
            if (saved) {
                const loadedArchive = JSON.parse(saved);
                setArchive(loadedArchive);
                // Verify files exist in background
                verifyArchiveFiles(loadedArchive);
            }
            // Load format preference
            const savedFormat = localStorage.getItem("ripvid-format");
            if (savedFormat === "mp3" || savedFormat === "mp4") {
                setDownloadFormat(savedFormat);
            }
            // Load quality preference
            const savedQuality = localStorage.getItem("ripvid-quality");
            if (savedQuality) {
                setQuality(savedQuality);
            }
        };

        initializeApp();
    }, []);

    useEffect(() => {
        // Click outside to close archive
        const handleClickOutside = (event: MouseEvent) => {
            if (
                archiveOpen &&
                archivePanelRef.current &&
                !archivePanelRef.current.contains(event.target as Node) &&
                !(event.target as Element).closest(".archive-toggle")
            ) {
                setArchiveOpen(false);
            }

            if (
                showSettings &&
                settingsPanelRef.current &&
                !settingsPanelRef.current.contains(event.target as Node) &&
                !(event.target as Element).closest(".settings-toggle")
            ) {
                setShowSettings(false);
            }
        };

        document.addEventListener("mousedown", handleClickOutside);
        return () =>
            document.removeEventListener("mousedown", handleClickOutside);
    }, [archiveOpen, showSettings]);

    useEffect(() => {
        if (
            status === "success" ||
            status === "error" ||
            status === "cancelled"
        ) {
            const timer = setTimeout(() => {
                setStatus("idle");
                setProgress(null);
                setUrl("");
                setProcessingStartTime(null);
                setProcessingElapsed(0);
                if (inputRef.current) {
                    inputRef.current.focus();
                }
            }, 3000);
            return () => clearTimeout(timer);
        }
    }, [status]);

    // Update elapsed time during processing phase
    useEffect(() => {
        if (status === "processing" && processingStartTime) {
            const interval = setInterval(() => {
                setProcessingElapsed(Math.floor((Date.now() - processingStartTime) / 1000));
            }, 1000);
            return () => clearInterval(interval);
        }
    }, [status, processingStartTime]);

    const detectPlatform = async (videoUrl: string) => {
        try {
            const detected = await invoke<string>("detect_platform", {
                url: videoUrl,
            });
            setPlatform(detected);
            return detected;
        } catch (error) {
            console.error("Failed to detect platform:", error);
            setPlatform(null);
            return null;
        }
    };

    const handleUrlChange = async (e: React.ChangeEvent<HTMLInputElement>) => {
        const newUrl = e.target.value;
        setUrl(newUrl);

        if (newUrl.trim()) {
            await detectPlatform(newUrl);
        } else {
            setPlatform(null);
        }
    };

    const getDownloadPath = async () => {
        const home = await homeDir();
        const formatFolder = downloadFormat.toUpperCase();
        const ripvidDir = await join(home, "ripVID", formatFolder);

        // Create directory if it doesn't exist
        await invoke("create_directory", { path: ripvidDir });

        const timestamp = new Date()
            .toISOString()
            .replace(/[:.]/g, "-")
            .slice(0, -5);
        const filename = `${platform}_${timestamp}.${downloadFormat}`;

        return await join(ripvidDir, filename);
    };

    const handleDownload = async () => {
        if (!url.trim() || !platform || isDownloading) return;

        console.log("Starting download:", {
            url,
            platform,
            format: downloadFormat,
            quality,
        });

        setIsDownloading(true);
        setStatus("downloading");
        setProgress(null);

        // Store download info for later use in completion handler
        downloadInfoRef.current = {
            url: url.trim(),
            platform: platform,
            format: downloadFormat,
        };

        try {
            const savePath = await getDownloadPath();
            console.log("Save path:", savePath);

            // Use different command based on format
            if (downloadFormat === "mp3") {
                console.log("Downloading as MP3...");
                const downloadId = await invoke<string>("download_audio", {
                    url: url.trim(),
                    outputPath: savePath,
                });
                console.log("Audio download started with ID:", downloadId);
            } else {
                console.log("Downloading as MP4 with quality:", quality);
                const downloadId = await invoke<string>("download_video", {
                    url: url.trim(),
                    outputPath: savePath,
                    quality: quality,
                });
                console.log("Video download started with ID:", downloadId);
            }

            // The actual completion and archive addition will be handled by the download-complete event
        } catch (error) {
            console.error("Failed to start download:", error);
            setStatus("error");
            setIsDownloading(false);
            setCurrentDownloadId(null);
            downloadInfoRef.current = null;
        }
    };

    const handleCancelDownload = async () => {
        if (!currentDownloadId) return;

        console.log("Cancelling download:", currentDownloadId);

        try {
            await invoke("cancel_download_command", {
                downloadId: currentDownloadId,
            });
            console.log("Download cancelled successfully");
        } catch (error) {
            console.error("Failed to cancel download:", error);
        }
    };

    const handleKeyPress = (e: React.KeyboardEvent<HTMLInputElement>) => {
        if (e.key === "Enter") {
            handleDownload();
        } else if (e.key === "Escape") {
            if (isDownloading) {
                handleCancelDownload();
            } else {
                setUrl("");
                setPlatform(null);
                setStatus("idle");
            }
        } else if (e.key === "Tab") {
            e.preventDefault();
            setArchiveOpen(!archiveOpen);
        }
    };

    const openFile = async (path: string, fileExists: boolean | undefined) => {
        // Don't attempt to open if we know the file doesn't exist
        if (fileExists === false) {
            console.warn("Cannot open file - file does not exist:", path);
            return;
        }

        try {
            // Open the file directly with the system's default application
            await invoke("open_file_directly", { path });
        } catch (error) {
            console.error("Failed to open file:", error);
            // Fallback: try to open the containing folder instead
            try {
                await invoke("open_file_location", { path });
            } catch (fallbackError) {
                console.error("Fallback to folder also failed:", fallbackError);
            }
        }
    };

    const deleteFromArchive = async (id: string) => {
        const item = archive.find((item) => item.id === id);
        if (!item) return;

        try {
            // Recycle the actual file
            await invoke("recycle_file", { path: item.path });

            // Remove from archive
            const newArchive = archive.filter((item) => item.id !== id);
            setArchive(newArchive);
            localStorage.setItem("ripvid-archive", JSON.stringify(newArchive));
        } catch (error) {
            console.error("Failed to recycle file:", error);
            // Still remove from archive even if file recycling fails
            const newArchive = archive.filter((item) => item.id !== id);
            setArchive(newArchive);
            localStorage.setItem("ripvid-archive", JSON.stringify(newArchive));
        }
    };

    const setupFolderStructure = async () => {
        try {
            const home = await homeDir();
            const ripvidDir = await join(home, "ripVID");
            const mp4Dir = await join(ripvidDir, "MP4");
            const mp3Dir = await join(ripvidDir, "MP3");

            // Create all directories
            await invoke("create_directory", { path: ripvidDir });
            await invoke("create_directory", { path: mp4Dir });
            await invoke("create_directory", { path: mp3Dir });

            console.log("Folder structure created successfully");
        } catch (error) {
            console.error("Failed to create folder structure:", error);
        }
    };

    // Verify if files in archive actually exist
    const verifyArchiveFiles = async (archiveItems: ArchiveItem[]) => {
        const updatedArchive = await Promise.all(
            archiveItems.map(async (item) => {
                try {
                    const exists = await invoke<boolean>("file_exists", {
                        path: item.path,
                    });
                    return { ...item, fileExists: exists };
                } catch (error) {
                    console.error("Failed to verify file:", item.path, error);
                    return { ...item, fileExists: false };
                }
            }),
        );

        setArchive(updatedArchive);
        localStorage.setItem("ripvid-archive", JSON.stringify(updatedArchive));
    };

    // Refresh archive by scanning actual download folders
    const refreshArchive = async () => {
        try {
            console.log("Refreshing archive from disk...");
            const files = await invoke<any[]>("scan_downloads_folder");

            if (files.length === 0) {
                console.log("No files found in downloads folder");
                return;
            }

            // Convert scanned files to archive items
            const scannedItems: ArchiveItem[] = files.map((file, index) => ({
                id: `scanned-${Date.now()}-${index}`,
                title: file.filename,
                url: "", // Unknown for scanned files
                platform: "unknown",
                date: new Date(
                    (file.modified || Date.now() / 1000) * 1000,
                ).toLocaleDateString(),
                path: file.path,
                format: file.format,
                fileExists: true,
            }));

            // Merge with existing archive, removing duplicates by path
            const existingPaths = new Set(
                archive.map((item) => item.path.toLowerCase()),
            );
            const newItems = scannedItems.filter(
                (item) => !existingPaths.has(item.path.toLowerCase()),
            );

            if (newItems.length > 0) {
                const mergedArchive = [...archive, ...newItems];
                setArchive(mergedArchive);
                localStorage.setItem(
                    "ripvid-archive",
                    JSON.stringify(mergedArchive),
                );
                console.log(
                    `Added ${newItems.length} files from disk to archive`,
                );
            } else {
                console.log("All disk files already in archive");
            }

            // Re-verify all files
            await verifyArchiveFiles(archive);
        } catch (error) {
            console.error("Failed to refresh archive:", error);
        }
    };

    const handleAcceptTerms = async () => {
        localStorage.setItem("ripvid-terms-accepted", "true");
        setShowTerms(false);
        await setupFolderStructure();
    };

    const handleDeclineTerms = () => {
        // Close the app if terms are declined
        const appWindow = getCurrentWebviewWindow();
        appWindow.close();
    };

    const toggleFormat = () => {
        const newFormat = downloadFormat === "mp4" ? "mp3" : "mp4";
        setDownloadFormat(newFormat);
        localStorage.setItem("ripvid-format", newFormat);
    };

    const handleQualityChange = (newQuality: string) => {
        setQuality(newQuality);
        localStorage.setItem("ripvid-quality", newQuality);
    };

    const getFilteredArchive = () => {
        if (archiveTab === "all") return archive;
        if (archiveTab === "video")
            return archive.filter((item) => item.format === "mp4");
        if (archiveTab === "audio")
            return archive.filter((item) => item.format === "mp3");
        return archive;
    };

    const getPlatformIcon = (size = 14) => {
        if (platform === "youtube") return <Youtube size={size} />;
        if (platform === "x") return <Globe size={size} />;
        return null;
    };

    // Estimate remaining conversion time
    // Heuristic: MP3 conversion typically takes ~15-30 seconds for a typical song
    // Video processing is generally faster (merging streams)
    const getEstimatedRemaining = () => {
        const isAudioConversion = processingMessage.includes("MP3") || processingMessage.includes("audio");
        // Estimate: audio conversion ~20s average, video merge ~10s average
        const estimatedTotal = isAudioConversion ? 25 : 12;
        const remaining = Math.max(0, estimatedTotal - processingElapsed);
        return remaining;
    };

    const getStatusContent = () => {
        if (status === "processing") {
            const remaining = getEstimatedRemaining();
            const showEstimate = remaining > 0 && processingElapsed < 60; // Don't show estimate after 60s

            return (
                <div className="processing-text">
                    <RefreshCw size={14} className="processing-spinner" />
                    <span>
                        {processingMessage}
                        {showEstimate ? ` ~${remaining}s` : processingElapsed > 0 ? ` (${processingElapsed}s)` : ""}
                    </span>
                </div>
            );
        }

        if (isDownloading && progress) {
            return (
                <div className="progress-text">
                    <span className="progress-platform">
                        {getPlatformIcon()}
                    </span>
                    <span className="progress-percent">
                        {Math.round(progress.percent)}%
                    </span>
                    <span className="progress-separator">•</span>
                    <span className="progress-speed">{progress.speed}</span>
                    <span className="progress-separator">•</span>
                    <span className="progress-eta">ETA {progress.eta}</span>
                </div>
            );
        }

        if (status === "success") {
            return <div className="success-text">Download complete</div>;
        }

        if (status === "error") {
            return <div className="error-text">Download failed</div>;
        }

        if (status === "cancelled") {
            return <div className="cancelled-text">Download cancelled</div>;
        }

        return null;
    };

    return (
        <>
            {showTerms && (
                <TermsAcceptance
                    onAccept={handleAcceptTerms}
                    onDecline={handleDeclineTerms}
                />
            )}
            <TitleBar />
            <UpdateChecker />
            <ShaderBackground
                speed={0.15}
                intensity={0.8}
                scale={1.8}
                opacity={0.6}
                enabled={true}
            />
            <div className="app">
                <div className="logo">
                    <span className="logo-text">rip</span>
                    <span className="logo-v">V</span>
                    <span className="logo-text">ID</span>
                </div>

                <button
                    className={`format-toggle ${downloadFormat}`}
                    onClick={toggleFormat}
                    aria-label={`Switch to ${downloadFormat === "mp4" ? "MP3" : "MP4"}`}
                >
                    <div className="format-toggle-inner">
                        <div className="format-option mp4">
                            <Play size={14} />
                        </div>
                        <div className="format-option mp3">
                            <Music size={14} />
                        </div>
                    </div>
                </button>

                {/* Settings Panel */}
                <div
                    ref={settingsPanelRef}
                    className={`settings-panel ${showSettings ? "open" : ""}`}
                >
                    <div className="settings-header">
                        <h3>Settings</h3>
                    </div>
                    <div className="settings-content">
                        {/* Video Quality - only shown for MP4 mode */}
                        {downloadFormat === "mp4" && (
                            <div className="setting-group">
                                <label>Video Quality</label>
                                <div className="quality-selector">
                                    {[
                                        "best",
                                        "1080p",
                                        "720p",
                                        "480p",
                                        "360p",
                                    ].map((q) => (
                                        <button
                                            key={q}
                                            className={`quality-option ${quality === q ? "active" : ""}`}
                                            onClick={() =>
                                                handleQualityChange(q)
                                            }
                                        >
                                            {q}
                                        </button>
                                    ))}
                                </div>
                            </div>
                        )}

                        {/* MP3 mode hint */}
                        {downloadFormat === "mp3" && (
                            <div className="setting-group">
                                <label>Audio Mode</label>
                                <div className="setting-hint">
                                    Downloads audio as MP3 at best quality.
                                    Use the toggle button (top-left) to switch to video mode.
                                </div>
                            </div>
                        )}

                        {/* Info section */}
                        <div className="setting-group">
                            <label>Download Location</label>
                            <div className="setting-info">~/ripVID/{downloadFormat.toUpperCase()}/</div>
                        </div>
                    </div>
                </div>

                <div className="input-container">
                    <div
                        className={`input-wrapper ${isDownloading ? "downloading" : ""}`}
                    >
                        <input
                            ref={inputRef}
                            type="url"
                            placeholder="Paste URL here..."
                            value={url}
                            onChange={handleUrlChange}
                            onKeyDown={handleKeyPress}
                            className="main-input"
                            disabled={isDownloading}
                            autoFocus
                        />
                        {!isDownloading ? (
                            <button
                                className="power-button"
                                onClick={handleDownload}
                                disabled={!url || isDownloading}
                                type="button"
                            >
                                <Power size={22} />
                            </button>
                        ) : (
                            <button
                                className="cancel-button"
                                onClick={handleCancelDownload}
                                type="button"
                                title="Cancel download (ESC)"
                            >
                                <XCircle size={22} />
                            </button>
                        )}
                    </div>
                    <div
                        className={`status-info ${status !== "idle" ? "active" : ""}`}
                    >
                        {getStatusContent()}
                    </div>
                </div>

                {/* Settings toggle - hidden when any panel is open */}
                <button
                    className={`settings-toggle ${showSettings || archiveOpen ? "hidden" : ""}`}
                    onClick={() => {
                        setShowSettings(true);
                        setArchiveOpen(false);
                    }}
                    aria-label="Open settings"
                >
                    ⚙
                </button>
                {/* Archive toggle - hidden when any panel is open */}
                <button
                    className={`archive-toggle ${showSettings || archiveOpen ? "hidden" : ""}`}
                    onClick={() => {
                        setArchiveOpen(true);
                        setShowSettings(false);
                    }}
                    aria-label="Open archive"
                >
                    <Save size={18} />
                </button>

                <div
                    ref={archivePanelRef}
                    className={`archive-panel ${archiveOpen ? "open" : ""}`}
                >
                    <div className="archive-header">
                        <div className="archive-tabs">
                            <button
                                className={`archive-tab ${archiveTab === "all" ? "active" : ""}`}
                                onClick={() => setArchiveTab("all")}
                                title={`All (${archive.length})`}
                            >
                                <Layers size={20} />
                            </button>
                            <span className="tab-divider">|</span>
                            <button
                                className={`archive-tab ${archiveTab === "video" ? "active" : ""}`}
                                onClick={() => setArchiveTab("video")}
                                title={`Videos (${archive.filter((i) => i.format === "mp4").length})`}
                            >
                                <Play size={20} />
                            </button>
                            <span className="tab-divider">|</span>
                            <button
                                className={`archive-tab ${archiveTab === "audio" ? "active" : ""}`}
                                onClick={() => setArchiveTab("audio")}
                                title={`Audio (${archive.filter((i) => i.format === "mp3").length})`}
                            >
                                <Music size={20} />
                            </button>
                        </div>
                        <button
                            className="archive-refresh-btn"
                            onClick={refreshArchive}
                            title="Refresh archive from disk"
                            aria-label="Refresh archive"
                        >
                            <RefreshCw size={16} />
                        </button>
                    </div>

                    {getFilteredArchive().length > 0 ? (
                        <div className="archive-list">
                            {getFilteredArchive().map((item) => (
                                <div
                                    key={item.id}
                                    className={`archive-item ${item.fileExists === false ? "missing-file" : ""}`}
                                >
                                    <div
                                        className="archive-item-content"
                                        onClick={() => openFile(item.path, item.fileExists)}
                                    >
                                        {item.fileExists === false && (
                                            <span title="File not found - may have been moved or deleted">
                                                <AlertCircle
                                                    size={14}
                                                    className="missing-file-icon"
                                                />
                                            </span>
                                        )}
                                        <span className="archive-item-name">
                                            {item.title}
                                        </span>
                                        <span
                                            className={`archive-item-type ${item.format === "mp3" ? "audio" : "video"}`}
                                        >
                                            {item.format?.toUpperCase()}
                                        </span>
                                        <span className="archive-item-date">
                                            {item.date}
                                        </span>
                                    </div>
                                    <button
                                        className="archive-item-delete"
                                        onClick={async (e) => {
                                            e.stopPropagation();
                                            await deleteFromArchive(item.id);
                                        }}
                                        aria-label="Delete from archive"
                                    >
                                        <X size={13} />
                                    </button>
                                </div>
                            ))}
                        </div>
                    ) : (
                        <div className="archive-empty">No downloads yet</div>
                    )}
                </div>
            </div>
        </>
    );
}

export default App;
