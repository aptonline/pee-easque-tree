import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Store } from "@tauri-apps/plugin-store";
import "./App.css";
import { FetchResult, PackageInfo, DownloadJob, ProgressInfo } from "./types";
import { Ps3WaveBackground } from "./components/Ps3WaveBackground";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Progress } from "@/components/ui/progress";
import {
  Drawer,
  DrawerContent,
  DrawerHeader,
  DrawerTitle,
} from "@/components/ui/drawer";
import { Label } from "@/components/ui/label";
import { Checkbox } from "@/components/ui/checkbox";
import { TypographyMuted } from "@/components/ui/typography";
import { Badge } from "@/components/ui/badge";
import { ScrollArea } from "@/components/ui/scroll-area";
import { X } from "lucide-react";

// Store will be initialized in the component

const PS3_THEMES = {
  classic: { name: "Classic Blue", color: "#00C8FF" },
  cyan: { name: "Cyan Wave", color: "#00FFFF" },
  purple: { name: "Purple Glow", color: "#9D4EDD" },
  green: { name: "Matrix Green", color: "#39FF14" },
  pink: { name: "Snow White", color: "#FFFFFF" },
  orange: { name: "Sunset Orange", color: "#FF6B00" },
  yellow: { name: "Electric Yellow", color: "#FFD700" },
  red: { name: "Crimson Red", color: "#DC143C" },
} as const;

type ThemeKey = keyof typeof PS3_THEMES;

const PS3_FACTS = [
  "The PS3 was the first gaming console to support Blu-ray discs.",
  "The original PS3 could run Linux through the 'Other OS' feature.",
  "The Cell processor in PS3 was also used in some supercomputers.",
  "PS3's XMB (XrossMediaBar) interface was inspired by Sony's PSP.",
  "The PS3 launched at $599 USD for the 60GB model in 2006.",
  "Over 87 million PS3 consoles were sold worldwide.",
  "The Last of Us on PS3 won over 200 Game of the Year awards.",
  "PS3 can output games at 1080p resolution.",
  "The PS3 Slim reduced power consumption by 34% compared to the original.",
  "Uncharted 2 showcased the PS3's graphical capabilities in 2009.",
  "PS3 had free online multiplayer through PlayStation Network.",
  "The SIXAXIS controller was the first PlayStation controller without rumble.",
  "Metal Gear Solid 4 was designed specifically for PS3's Cell processor.",
  "Journey on PS3 won multiple awards for its innovative multiplayer.",
  "PS3 could be used as a media server with PlayMemories.",
  "LittleBigPlanet allowed players to create and share their own levels.",
  "The PS3 Super Slim model was 25% lighter than the Slim.",
  "Demon's Souls pioneered the 'Souls-like' genre on PS3.",
  "PS3 supported 3D gaming with compatible TVs and games.",
  "Gran Turismo 5 featured over 1,000 cars at launch.",
  "PS3's Cell processor had 8 cores running at 3.2 GHz.",
  "The PlayStation Store launched day one with the PS3.",
  "Resistance: Fall of Man was a PS3 launch title in 2006.",
  "PS3 could play PS1 games but PS2 compatibility was limited.",
  "Infamous let players choose between hero or villain paths.",
  "The PS3 was one of the most affordable Blu-ray players at launch.",
  "Heavy Rain featured an interactive drama with multiple endings.",
  "Killzone 2 took 4 years to develop for the PS3.",
  "PS3's GPU was developed by NVIDIA and called the RSX 'Reality Synthesizer'.",
  "God of War III showcased some of the largest bosses in gaming history.",
];

function App() {
  const [titleId, setTitleId] = useState("");
  const [searchResult, setSearchResult] = useState<FetchResult | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [serverOnline, setServerOnline] = useState<boolean | null>(null);
  const [downloadPath, setDownloadPath] = useState("");
  const [downloads, setDownloads] = useState<DownloadJob[]>([]);
  const [multiPart, setMultiPart] = useState(true);
  const [showSettings, setShowSettings] = useState(false);
  const [themeColor, setThemeColor] = useState<ThemeKey>("classic");
  const [ps3Fact, setPs3Fact] = useState("");

  useEffect(() => {
    checkServerStatus();
    loadSettings();
    // Pick a random PS3 fact on load
    const randomFact = PS3_FACTS[Math.floor(Math.random() * PS3_FACTS.length)];
    setPs3Fact(randomFact);
  }, []);

  useEffect(() => {
    const interval = setInterval(() => {
      updateDownloadProgress();
    }, 500);

    return () => clearInterval(interval);
  }, [downloads]);

  // Periodically check server status every 30 seconds
  useEffect(() => {
    const interval = setInterval(() => {
      checkServerStatus();
    }, 30000);

    return () => clearInterval(interval);
  }, []);

  useEffect(() => {
    if (!titleId.trim()) {
      setSearchResult(null);
      setError(null);
    }
  }, [titleId]);

  // Set CSS variables on document root for global access (including Portals)
  useEffect(() => {
    const root = document.documentElement;
    const color = PS3_THEMES[themeColor].color;
    const rgb = color.replace('#', '').match(/.{2}/g)?.map(x => parseInt(x, 16)).join(', ') || '0, 200, 255';

    root.style.setProperty('--theme-color', color);
    root.style.setProperty('--theme-color-rgb', rgb);
  }, [themeColor]);

  const checkServerStatus = async () => {
    try {
      const online = await invoke<boolean>("check_server_status");
      setServerOnline(online);
    } catch (err) {
      console.error("Failed to check server status:", err);
      setServerOnline(false);
    }
  };

  const loadSettings = async () => {
    try {
      console.log("[Settings] Loading settings from store...");
      const store = await Store.load("settings.json");

      // Load theme color from store
      const savedTheme = await store.get<string>("theme");
      console.log("[Settings] Loaded theme:", savedTheme);
      if (savedTheme && savedTheme in PS3_THEMES) {
        setThemeColor(savedTheme as ThemeKey);
        console.log("[Settings] Applied theme:", savedTheme);
      } else {
        console.log("[Settings] No valid theme found, using default");
      }

      // Load multipart setting
      const savedMultiPart = await store.get<boolean>("multiPart");
      console.log("[Settings] Loaded multiPart:", savedMultiPart);
      if (savedMultiPart !== null && savedMultiPart !== undefined) {
        setMultiPart(savedMultiPart);
        console.log("[Settings] Applied multiPart:", savedMultiPart);
      } else {
        console.log("[Settings] No multiPart setting found, using default");
      }

      // Load download path from store
      const savedPath = await store.get<string>("downloadPath");
      console.log("[Settings] Loaded downloadPath:", savedPath);
      if (savedPath) {
        setDownloadPath(savedPath);
        console.log("[Settings] Applied downloadPath:", savedPath);
      } else {
        // Fallback to default download directory
        const path = await invoke<string>("get_default_download_path");
        setDownloadPath(path);
        console.log("[Settings] Using default downloadPath:", path);
      }
      console.log("[Settings] Settings loaded successfully");
    } catch (err) {
      console.error("Failed to load settings:", err);
      // Fallback to defaults on error
      try {
        const path = await invoke<string>("get_default_download_path");
        setDownloadPath(path);
      } catch (pathErr) {
        console.error("Failed to get default path:", pathErr);
      }
    }
  };

  const saveThemeColor = async (theme: ThemeKey) => {
    console.log("[Settings] Saving theme:", theme);
    setThemeColor(theme);
    try {
      const store = await Store.load("settings.json");
      await store.set("theme", theme);
      await store.save();
      console.log("[Settings] Theme saved successfully:", theme);
    } catch (err) {
      console.error("Failed to save theme:", err);
    }
  };

  const saveDownloadPath = async (path: string) => {
    console.log("[Settings] Saving downloadPath:", path);
    setDownloadPath(path);
    try {
      const store = await Store.load("settings.json");
      await store.set("downloadPath", path);
      await store.save();
      console.log("[Settings] downloadPath saved successfully:", path);
    } catch (err) {
      console.error("Failed to save download path:", err);
    }
  };

  const saveMultiPart = async (enabled: boolean) => {
    console.log("[Settings] Saving multiPart:", enabled);
    setMultiPart(enabled);
    try {
      const store = await Store.load("settings.json");
      await store.set("multiPart", enabled);
      await store.save();
      console.log("[Settings] multiPart saved successfully:", enabled);
    } catch (err) {
      console.error("Failed to save multiPart setting:", err);
    }
  };

  const searchUpdates = async (e: React.FormEvent) => {
    e.preventDefault();

    if (!titleId.trim()) {
      setSearchResult(null);
      setError(null);
      return;
    }

    setLoading(true);
    setError(null);
    setSearchResult(null);

    try {
      const result = await invoke<FetchResult>("fetch_updates", { titleId });
      setSearchResult(result);
      if (result.error) {
        setError(result.error);
      }
    } catch (err) {
      setError(String(err));
    } finally {
      setLoading(false);
    }
  };

  const pickDownloadDirectory = async () => {
    try {
      const path = await invoke<string | null>("pick_download_directory");
      if (path) {
        await saveDownloadPath(path);
      }
    } catch (err) {
      console.error("Failed to pick directory:", err);
    }
  };

  const startDownload = async (pkg: PackageInfo) => {
    if (!searchResult) return;

    try {
      const jobId = await invoke<string>("start_download", {
        url: pkg.url,
        filename: pkg.filename,
        downloadPath: downloadPath,
        gameTitle: searchResult.game_title,
        titleId: searchResult.cleaned_title_id,
        multiPart: multiPart,
      });

      setDownloads((prev) => [
        ...prev,
        {
          jobId,
          package: pkg,
          progress: null,
        },
      ]);
    } catch (err) {
      setError(`Failed to start download: ${err}`);
    }
  };

  const cancelDownload = async (jobId: string) => {
    try {
      await invoke("cancel_download", { jobId });
      setDownloads((prev) => prev.filter((d) => d.jobId !== jobId));
    } catch (err) {
      setError(`Failed to cancel download: ${err}`);
    }
  };

  const updateDownloadProgress = async () => {
    const updatedDownloads = [...downloads];
    let hasChanges = false;

    for (let i = 0; i < updatedDownloads.length; i++) {
      const download = updatedDownloads[i];
      if (!download.progress?.done) {
        try {
          const progress = await invoke<ProgressInfo>("get_download_progress", {
            jobId: download.jobId,
          });
          updatedDownloads[i] = { ...download, progress };
          hasChanges = true;

          if (progress.done && !progress.error) {
            await invoke("remove_download_job", { jobId: download.jobId });
          }
        } catch (err) {
          console.error("Failed to get progress:", err);
        }
      }
    }

    if (hasChanges) {
      setDownloads(updatedDownloads);
    }
  };

  return (
    <div
      className="app dark"
      style={{
        '--theme-color': PS3_THEMES[themeColor].color,
        '--theme-color-rgb': PS3_THEMES[themeColor].color.replace('#', '').match(/.{2}/g)?.map(x => parseInt(x, 16)).join(', ') || '0, 200, 255'
      } as React.CSSProperties}
    >
      <div className={showSettings ? "app-content blur-content" : "app-content"}>
      <header>
        <Ps3WaveBackground waveColor={PS3_THEMES[themeColor].color} />
        <div className="header-top">
          <div className="header-brand">
            <img src="/pee-esque-tree-logo-small.png" alt="Pee-esque-tree Logo" className="header-logo" />
            <div className="header-titles">
              <h1>Pee-esque-tree</h1>
              <h2>Game Update Downloader</h2>
            </div>
          </div>
          <div className="header-right">
            <Badge variant={serverOnline ? "success" : serverOnline === null ? "secondary" : "destructive"}>
              {serverOnline === null ? "Checking..." : serverOnline ? "Online" : "Offline"}
            </Badge>
            <Button
              variant="ghost"
              size="icon"
              className="settings-icon-btn"
              onClick={() => setShowSettings(true)}
              title="Settings"
            >
              <svg
                width="24"
                height="24"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                strokeWidth="2"
                strokeLinecap="round"
                strokeLinejoin="round"
              >
                <path d="M12.22 2h-.44a2 2 0 0 0-2 2v.18a2 2 0 0 1-1 1.73l-.43.25a2 2 0 0 1-2 0l-.15-.08a2 2 0 0 0-2.73.73l-.22.38a2 2 0 0 0 .73 2.73l.15.1a2 2 0 0 1 1 1.72v.51a2 2 0 0 1-1 1.74l-.15.09a2 2 0 0 0-.73 2.73l.22.38a2 2 0 0 0 2.73.73l.15-.08a2 2 0 0 1 2 0l.43.25a2 2 0 0 1 1 1.73V20a2 2 0 0 0 2 2h.44a2 2 0 0 0 2-2v-.18a2 2 0 0 1 1-1.73l.43-.25a2 2 0 0 1 2 0l.15.08a2 2 0 0 0 2.73-.73l.22-.39a2 2 0 0 0-.73-2.73l-.15-.08a2 2 0 0 1-1-1.74v-.5a2 2 0 0 1 1-1.74l.15-.09a2 2 0 0 0 .73-2.73l-.22-.38a2 2 0 0 0-2.73-.73l-.15.08a2 2 0 0 1-2 0l-.43-.25a2 2 0 0 1-1-1.73V4a2 2 0 0 0-2-2z" />
                <circle cx="12" cy="12" r="3" />
              </svg>
            </Button>
          </div>
        </div>

        <div className="header-search">
          <form onSubmit={searchUpdates} className="search-form">
            <div className="relative flex-1 max-w-md">
              <Input
                type="text"
                value={titleId}
                onChange={(e) => setTitleId(e.target.value.toUpperCase())}
                placeholder="Enter a Title ID (e.g., BLES00799)"
                disabled={loading}
                className="pr-8"
              />
              {titleId && (
                <Button
                  type="button"
                  variant="ghost"
                  size="icon"
                  className="absolute right-0 top-0 h-full px-3 py-2 hover:bg-transparent"
                  onClick={() => setTitleId("")}
                >
                  <X className="h-4 w-4" />
                </Button>
              )}
            </div>
            <Button type="submit" disabled={loading || !titleId}>
              {loading ? "Searching..." : "Search"}
            </Button>
            <span className="header-hint">
              Find game IDs at{" "}
              <a
                href="https://serialstation.com/"
                target="_blank"
                rel="noopener noreferrer"
              >
                SerialStation.com
              </a>
            </span>
          </form>
        </div>
      </header>

      <Drawer open={showSettings} onOpenChange={setShowSettings}>
        <DrawerContent className="max-h-[85vh]">
          <DrawerHeader>
            <DrawerTitle>Settings</DrawerTitle>
          </DrawerHeader>
          <div className="overflow-y-auto px-4 pb-6 space-y-6">
            <div className="space-y-3">
              <Label>Download Location</Label>
              <div className="flex w-full items-center space-x-2">
                <Input
                  type="text"
                  value={downloadPath}
                  onChange={(e) => setDownloadPath(e.target.value)}
                  onBlur={(e) => saveDownloadPath(e.target.value)}
                  placeholder="Download directory path"
                />
                <Button type="button" onClick={pickDownloadDirectory} variant="secondary">
                  Browse
                </Button>
              </div>
              <TypographyMuted>
                Files will be saved in a subfolder named: "Game Title (TITLE_ID)"
              </TypographyMuted>
            </div>

            <div className="space-y-3">
              <div className="flex items-center space-x-3">
                <Checkbox
                  id="multipart"
                  checked={multiPart}
                  onCheckedChange={(checked) => saveMultiPart(checked as boolean)}
                />
                <Label htmlFor="multipart" className="cursor-pointer font-normal">
                  Multi-part download (4 parts, faster)
                </Label>
              </div>
            </div>

            <div className="space-y-3">
              <Label>Wave Theme</Label>
              <div className="grid grid-cols-2 gap-2">
                {(Object.keys(PS3_THEMES) as ThemeKey[]).map((key) => (
                  <Button
                    key={key}
                    variant={themeColor === key ? "default" : "outline"}
                    className="justify-start gap-2 h-auto py-2.5 px-3"
                    onClick={() => saveThemeColor(key)}
                    data-theme-active={themeColor === key}
                  >
                    <span
                      className="theme-color-preview"
                      style={{ backgroundColor: PS3_THEMES[key].color }}
                    />
                    <span className="text-sm">{PS3_THEMES[key].name}</span>
                  </Button>
                ))}
              </div>
            </div>
          </div>
        </DrawerContent>
      </Drawer>

      <main className="container">
        {error && <div className="error">{error}</div>}

        {!searchResult && !error && (
          <div className="empty-state">
            <p className="did-you-know">
              Did you know...
              <br />
              {ps3Fact}
            </p>
            <p>To get started make a search</p>
          </div>
        )}

        {searchResult && !searchResult.error && (
          <Card className="results-section">
            <CardHeader>
              <CardTitle>
                {searchResult.game_title} ({searchResult.cleaned_title_id})
              </CardTitle>
            </CardHeader>
            <CardContent>
              {searchResult.results.length === 0 ? (
                <p className="no-results">No updates found for this title.</p>
              ) : (
                <ScrollArea className="max-h-[400px] w-full pr-4">
                  <div className="updates-list">
                    {searchResult.results.map((pkg, index) => (
                      <Card key={index} className="update-card">
                      <CardContent className="p-4">
                        <div className="update-info">
                          <div className="update-version">
                            <TypographyMuted className="inline">Version:</TypographyMuted> {pkg.version}
                          </div>
                          <div className="update-detail">
                            <TypographyMuted className="inline">System Version:</TypographyMuted> {pkg.system_ver}
                          </div>
                          <div className="update-detail">
                            <TypographyMuted className="inline">Size:</TypographyMuted> {pkg.size_human}
                          </div>
                          <div className="update-detail">
                            <TypographyMuted className="inline">Filename:</TypographyMuted> {pkg.filename}
                          </div>
                          <div className="update-detail sha">
                            <TypographyMuted className="inline">SHA1:</TypographyMuted> <code>{pkg.sha1 || 'N/A'}</code>
                          </div>
                        </div>
                        <Button
                          onClick={() => startDownload(pkg)}
                          disabled={downloads.some(
                            (d) =>
                              d.package.filename === pkg.filename && !d.progress?.done
                          )}
                        >
                          Download
                        </Button>
                      </CardContent>
                    </Card>
                  ))}
                  </div>
                </ScrollArea>
              )}
            </CardContent>
          </Card>
        )}
      </main>

      {downloads.length > 0 && (
        <div className="floating-downloads">
          {downloads.map((download) => (
            <Card key={download.jobId} className="floating-download-item">
              <CardContent className="p-3">
                <div className="floating-download-info">
                  <span className="floating-download-name">{download.package.filename}</span>
                  {download.progress && (
                    <span className="floating-download-stats">
                      {download.progress.done ? (
                        <span className={download.progress.error ? "error" : "success"}>
                          {download.progress.error || "Complete"}
                        </span>
                      ) : (
                        <>
                          {download.progress.percent.toFixed(0)}% • {download.progress.speed_human}
                        </>
                      )}
                    </span>
                  )}
                </div>
                <Progress
                  value={download.progress?.percent || 0}
                  className="floating-progress-bar"
                />
                {!download.progress?.done && (
                  <Button
                    variant="ghost"
                    size="icon"
                    className="floating-cancel-btn"
                    onClick={() => cancelDownload(download.jobId)}
                    title="Cancel download"
                  >
                    ✕
                  </Button>
                )}
                {download.progress?.done && !download.progress?.error && (
                  <Button
                    variant="ghost"
                    size="icon"
                    className="floating-close-btn"
                    onClick={() => setDownloads((prev) => prev.filter((d) => d.jobId !== download.jobId))}
                    title="Close"
                  >
                    ✕
                  </Button>
                )}
              </CardContent>
            </Card>
          ))}
        </div>
      )}
      </div>
    </div>
  );
}

export default App;
