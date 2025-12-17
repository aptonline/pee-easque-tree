export interface PackageInfo {
  version: string;
  system_ver: string;
  size_bytes: number;
  size_human: string;
  url: string;
  sha1: string;
  filename: string;
}

export interface FetchResult {
  results: PackageInfo[];
  error: string | null;
  game_title: string;
  cleaned_title_id: string;
}

export interface ProgressInfo {
  filename: string | null;
  total: number;
  downloaded: number;
  percent: number;
  speed_bytes_per_sec: number;
  speed_human: string;
  done: boolean;
  error: string | null;
}

export interface DownloadJob {
  jobId: string;
  package: PackageInfo;
  progress: ProgressInfo | null;
}
