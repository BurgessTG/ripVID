import { useEffect, useState } from 'react';
import { check } from '@tauri-apps/plugin-updater';
import { relaunch } from '@tauri-apps/plugin-process';
import { Button } from './ui/button';
import { X, Download } from 'lucide-react';

interface DownloadEvent {
  event: 'Started' | 'Progress' | 'Finished';
  data: {
    contentLength?: number;
    chunkLength?: number;
  };
}

interface UpdateInfo {
  version: string;
  body?: string;
  available: boolean;
  downloadAndInstall: (callback: (event: DownloadEvent) => void) => Promise<void>;
}

export function UpdateChecker() {
  const [updateAvailable, setUpdateAvailable] = useState(false);
  const [updateInfo, setUpdateInfo] = useState<UpdateInfo | null>(null);
  const [downloading, setDownloading] = useState(false);
  const [progress, setProgress] = useState(0);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    checkForUpdates();
    // Check for updates every 30 minutes
    const interval = setInterval(checkForUpdates, 30 * 60 * 1000);
    return () => clearInterval(interval);
  }, []);

  async function checkForUpdates() {
    try {
      const update = await check();
      if (update?.available) {
        setUpdateAvailable(true);
        setUpdateInfo(update as UpdateInfo);
      }
    } catch (_error) {
      // Failed to check for updates
    }
  }

  async function installUpdate() {
    if (!updateInfo) return;

    try {
      setDownloading(true);
      setError(null);

      // Download and install
      let downloaded = 0;
      let contentLength = 0;

      await updateInfo.downloadAndInstall((event: DownloadEvent) => {
        switch (event.event) {
          case 'Started':
            contentLength = event.data.contentLength || 0;
            break;
          case 'Progress':
            downloaded += event.data.chunkLength || 0;
            if (contentLength > 0) {
              const percentage = (downloaded / contentLength) * 100;
              setProgress(Math.round(percentage));
            }
            break;
          case 'Finished':
            // Download complete
            break;
        }
      });

      // Restart the app
      await relaunch();
    } catch (error) {
      setError(error instanceof Error ? error.message : String(error));
      setDownloading(false);
    }
  }

  if (!updateAvailable) return null;

  return (
    <div className="fixed bottom-4 right-4 max-w-sm bg-gray-900 border border-purple-500/30 rounded-lg shadow-xl p-4 z-50">
      <div className="flex justify-between items-start mb-2">
        <h3 className="text-white font-semibold">Update Available</h3>
        <button
          onClick={() => setUpdateAvailable(false)}
          className="text-gray-400 hover:text-white transition-colors"
        >
          <X size={16} />
        </button>
      </div>

      <p className="text-gray-300 text-sm mb-3">
        Version {updateInfo?.version} is ready to install
      </p>

      {error && (
        <p className="text-red-400 text-xs mb-2">{error}</p>
      )}

      {downloading ? (
        <div className="space-y-2">
          <div className="w-full bg-gray-800 rounded-full h-2">
            <div
              className="bg-purple-500 h-2 rounded-full transition-all duration-300"
              style={{ width: `${progress}%` }}
            />
          </div>
          <p className="text-gray-400 text-xs text-center">{progress}%</p>
        </div>
      ) : (
        <div className="flex gap-2">
          <Button
            onClick={installUpdate}
            className="flex-1 bg-purple-600 hover:bg-purple-700"
            size="sm"
          >
            <Download size={14} className="mr-1" />
            Install Now
          </Button>
          <Button
            onClick={() => setUpdateAvailable(false)}
            variant="outline"
            size="sm"
            className="flex-1"
          >
            Later
          </Button>
        </div>
      )}
    </div>
  );
}