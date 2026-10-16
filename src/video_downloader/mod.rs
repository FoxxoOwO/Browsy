use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MediaKind {
    Mp4,
    Webm,
    HlsPlaylist,  // .m3u8
    DashManifest, // .mpd
    Audio,
    OtherVideo,
}

impl MediaKind {
    pub fn label(&self) -> &'static str {
        match self {
            MediaKind::Mp4 => "MP4 Video",
            MediaKind::Webm => "WebM Video",
            MediaKind::HlsPlaylist => "HLS Stream (m3u8)",
            MediaKind::DashManifest => "DASH Stream (mpd)",
            MediaKind::Audio => "Audio Stream",
            MediaKind::OtherVideo => "Video",
        }
    }

    pub fn extension(&self) -> &'static str {
        match self {
            MediaKind::Mp4 => "mp4",
            MediaKind::Webm => "webm",
            MediaKind::HlsPlaylist => "m3u8",
            MediaKind::DashManifest => "mpd",
            MediaKind::Audio => "mp3",
            MediaKind::OtherVideo => "mp4",
        }
    }
}

/// Detected video/audio media stream
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectedStream {
    pub id: u64,
    pub stream_url: String,
    pub page_url: String,
    pub page_title: String,
    pub kind: MediaKind,
    pub size_estimate_bytes: Option<u64>,
    pub detected_at: chrono::DateTime<chrono::Local>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DownloadState {
    Queued,
    Downloading {
        bytes_downloaded: u64,
        total_bytes: Option<u64>,
        percentage: f32,
        speed_kbps: f64,
    },
    Completed {
        file_path: PathBuf,
    },
    Failed {
        error: String,
    },
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadTask {
    pub id: u64,
    pub stream_id: Option<u64>,
    pub url: String,
    pub filename: String,
    pub destination_path: PathBuf,
    pub state: DownloadState,
    pub is_ytdlp: bool,
    pub started_at: chrono::DateTime<chrono::Local>,
}

pub struct VideoDownloader {
    streams: Mutex<HashMap<u64, DetectedStream>>,
    downloads: Arc<Mutex<HashMap<u64, DownloadTask>>>,
    next_id: Mutex<u64>,
    ytdlp_available: Mutex<Option<bool>>,
    default_download_dir: PathBuf,
}

impl VideoDownloader {
    pub fn new() -> Self {
        let download_dir = dirs_or_fallback_download_dir();

        Self {
            streams: Mutex::new(HashMap::new()),
            downloads: Arc::new(Mutex::new(HashMap::new())),
            next_id: Mutex::new(1),
            ytdlp_available: Mutex::new(None),
            default_download_dir: download_dir,
        }
    }

    /// Sniff a URL from network requests or DOM to check if it represents downloadable media
    pub fn sniff_url(&self, url: &str, page_url: &str, page_title: &str) -> Option<DetectedStream> {
        let kind = classify_media_url(url)?;

        let mut streams = self.streams.lock().unwrap();
        // Avoid duplicate detections of same stream URL on same page
        if let Some(existing) = streams.values().find(|s| s.stream_url == url) {
            return Some(existing.clone());
        }

        let id = {
            let mut id_lock = self.next_id.lock().unwrap();
            let cur = *id_lock;
            *id_lock += 1;
            cur
        };

        let stream = DetectedStream {
            id,
            stream_url: url.to_string(),
            page_url: page_url.to_string(),
            page_title: if page_title.is_empty() {
                "Web Video".to_string()
            } else {
                page_title.to_string()
            },
            kind,
            size_estimate_bytes: None,
            detected_at: chrono::Local::now(),
        };

        streams.insert(id, stream.clone());
        Some(stream)
    }

    /// Get all detected streams matching the current tab's URL
    pub fn streams_for_page(&self, page_url: &str) -> Vec<DetectedStream> {
        let streams = self.streams.lock().unwrap();
        streams
            .values()
            .filter(|s| s.page_url == page_url || page_url.is_empty())
            .cloned()
            .collect()
    }

    /// Total count of detected streams
    pub fn total_detected(&self) -> usize {
        self.streams.lock().unwrap().len()
    }

    /// Check if yt-dlp is installed and available
    pub fn check_ytdlp_available(&self) -> bool {
        let mut cached = self.ytdlp_available.lock().unwrap();
        if let Some(avail) = *cached {
            return avail;
        }

        let is_avail = std::process::Command::new("yt-dlp")
            .arg("--version")
            .output()
            .is_ok();

        *cached = Some(is_avail);
        is_avail
    }

    /// Start a direct download of a detected stream
    pub fn start_download(&self, stream_id: u64) -> Result<u64, String> {
        let stream = {
            let streams = self.streams.lock().unwrap();
            streams.get(&stream_id).cloned().ok_or("Stream not found")?
        };

        let download_id = {
            let mut id_lock = self.next_id.lock().unwrap();
            let cur = *id_lock;
            *id_lock += 1;
            cur
        };

        // Determine sanitize filename
        let safe_title = sanitize_filename(&stream.page_title);
        let filename = format!("{}_{}.{}", safe_title, download_id, stream.kind.extension());
        let dest_path = self.default_download_dir.join(&filename);

        // Ensure directory exists
        let _ = std::fs::create_dir_all(&self.default_download_dir);

        let task = DownloadTask {
            id: download_id,
            stream_id: Some(stream_id),
            url: stream.stream_url.clone(),
            filename,
            destination_path: dest_path.clone(),
            state: DownloadState::Queued,
            is_ytdlp: false,
            started_at: chrono::Local::now(),
        };

        self.downloads.lock().unwrap().insert(download_id, task);

        // Launch async downloader task
        let downloads_arc = self.downloads.clone();
        let stream_url = stream.stream_url.clone();

        tokio::spawn(async move {
            run_direct_download(download_id, stream_url, dest_path, downloads_arc).await;
        });

        Ok(download_id)
    }

    /// Start a download using yt-dlp for complex sites (YouTube, Vimeo, Twitch, etc.)
    pub fn start_ytdlp_download(&self, page_url: &str, title: &str) -> Result<u64, String> {
        let download_id = {
            let mut id_lock = self.next_id.lock().unwrap();
            let cur = *id_lock;
            *id_lock += 1;
            cur
        };

        let safe_title = sanitize_filename(title);
        let filename = format!("{}_%(title)s.%(ext)s", safe_title);
        let _dest_path = self.default_download_dir.join(&filename);
        let _ = std::fs::create_dir_all(&self.default_download_dir);

        let task = DownloadTask {
            id: download_id,
            stream_id: None,
            url: page_url.to_string(),
            filename: format!("{}_ytdlp", safe_title),
            destination_path: self.default_download_dir.clone(),
            state: DownloadState::Downloading {
                bytes_downloaded: 0,
                total_bytes: None,
                percentage: 0.0,
                speed_kbps: 0.0,
            },
            is_ytdlp: true,
            started_at: chrono::Local::now(),
        };

        self.downloads.lock().unwrap().insert(download_id, task);

        let downloads_arc = self.downloads.clone();
        let target_url = page_url.to_string();
        let output_template = self.default_download_dir.join("%(title)s.%(ext)s").to_string_lossy().to_string();

        tokio::spawn(async move {
            let output = std::process::Command::new("yt-dlp")
                .arg("-f")
                .arg("bestvideo+bestaudio/best")
                .arg("-o")
                .arg(&output_template)
                .arg(&target_url)
                .output();

            let mut map = downloads_arc.lock().unwrap();
            if let Some(t) = map.get_mut(&download_id) {
                match output {
                    Ok(out) if out.status.success() => {
                        t.state = DownloadState::Completed {
                            file_path: PathBuf::from(&output_template),
                        };
                    }
                    Ok(out) => {
                        let err_msg = String::from_utf8_lossy(&out.stderr).to_string();
                        t.state = DownloadState::Failed { error: err_msg };
                    }
                    Err(e) => {
                        t.state = DownloadState::Failed {
                            error: format!("Failed to run yt-dlp: {}", e),
                        };
                    }
                }
            }
        });

        Ok(download_id)
    }

    /// List all downloads
    pub fn all_downloads(&self) -> Vec<DownloadTask> {
        let mut list: Vec<_> = self.downloads.lock().unwrap().values().cloned().collect();
        list.sort_by(|a, b| b.started_at.cmp(&a.started_at));
        list
    }

    pub fn download_dir(&self) -> &Path {
        &self.default_download_dir
    }
}

impl Default for VideoDownloader {
    fn default() -> Self {
        Self::new()
    }
}

/// Direct async HTTP chunked download
async fn run_direct_download(
    task_id: u64,
    url: String,
    dest_path: PathBuf,
    downloads: Arc<Mutex<HashMap<u64, DownloadTask>>>,
) {
    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) Browsy/1.0")
        .build();

    let client = match client {
        Ok(c) => c,
        Err(e) => {
            let mut map = downloads.lock().unwrap();
            if let Some(t) = map.get_mut(&task_id) {
                t.state = DownloadState::Failed {
                    error: format!("HTTP Client error: {}", e),
                };
            }
            return;
        }
    };

    let response = match client.get(&url).send().await {
        Ok(r) => r,
        Err(e) => {
            let mut map = downloads.lock().unwrap();
            if let Some(t) = map.get_mut(&task_id) {
                t.state = DownloadState::Failed {
                    error: format!("Request failed: {}", e),
                };
            }
            return;
        }
    };

    let total_bytes = response.content_length();
    let mut file = match tokio::fs::File::create(&dest_path).await {
        Ok(f) => f,
        Err(e) => {
            let mut map = downloads.lock().unwrap();
            if let Some(t) = map.get_mut(&task_id) {
                t.state = DownloadState::Failed {
                    error: format!("File creation failed: {}", e),
                };
            }
            return;
        }
    };

    use tokio::io::AsyncWriteExt;
    let mut downloaded = 0u64;
    let start_time = std::time::Instant::now();
    let mut response = response;

    loop {
        match response.chunk().await {
            Ok(Some(chunk)) => {
                if let Err(e) = file.write_all(&chunk).await {
                    let mut map = downloads.lock().unwrap();
                    if let Some(t) = map.get_mut(&task_id) {
                        t.state = DownloadState::Failed {
                            error: format!("Write error: {}", e),
                        };
                    }
                    return;
                }
                downloaded += chunk.len() as u64;

                let elapsed_secs = start_time.elapsed().as_secs_f64().max(0.001);
                let speed_kbps = (downloaded as f64 / 1024.0) / elapsed_secs;
                let percentage = total_bytes
                    .map(|tot| (downloaded as f32 / tot as f32) * 100.0)
                    .unwrap_or(0.0);

                let mut map = downloads.lock().unwrap();
                if let Some(t) = map.get_mut(&task_id) {
                    t.state = DownloadState::Downloading {
                        bytes_downloaded: downloaded,
                        total_bytes,
                        percentage,
                        speed_kbps,
                    };
                }
            }
            Ok(None) => {
                // EOF reached successfully
                break;
            }
            Err(e) => {
                let mut map = downloads.lock().unwrap();
                if let Some(t) = map.get_mut(&task_id) {
                    t.state = DownloadState::Failed {
                        error: format!("Stream error: {}", e),
                    };
                }
                return;
            }
        }
    }

    let _ = file.flush().await;

    let mut map = downloads.lock().unwrap();
    if let Some(t) = map.get_mut(&task_id) {
        t.state = DownloadState::Completed {
            file_path: dest_path,
        };
    }
}

/// Classify URL into MediaKind based on extension or signature
fn classify_media_url(url: &str) -> Option<MediaKind> {
    let lower = url.to_lowercase();
    let path = lower.split('?').next().unwrap_or(&lower);

    if path.ends_with(".mp4") || path.ends_with(".m4v") {
        Some(MediaKind::Mp4)
    } else if path.ends_with(".webm") {
        Some(MediaKind::Webm)
    } else if path.ends_with(".m3u8") || lower.contains("hls") || lower.contains(".m3u8?") {
        Some(MediaKind::HlsPlaylist)
    } else if path.ends_with(".mpd") || lower.contains("dash") || lower.contains(".mpd?") {
        Some(MediaKind::DashManifest)
    } else if path.ends_with(".mp3") || path.ends_with(".aac") || path.ends_with(".m4a") || path.ends_with(".ogg") {
        Some(MediaKind::Audio)
    } else if path.ends_with(".mkv") || path.ends_with(".flv") || path.ends_with(".avi") || path.ends_with(".mov") {
        Some(MediaKind::OtherVideo)
    } else {
        None
    }
}

fn sanitize_filename(name: &str) -> String {
    let sanitized: String = name
        .chars()
        .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c } else { '_' })
        .take(50)
        .collect();

    if sanitized.trim().is_empty() {
        "video".to_string()
    } else {
        sanitized
    }
}

fn dirs_or_fallback_download_dir() -> PathBuf {
    if let Some(user_dirs) = std::env::var_os("USERPROFILE") {
        let path = PathBuf::from(user_dirs).join("Downloads").join("Browsy");
        return path;
    }
    PathBuf::from("./downloads")
}
