use browsy::video_downloader::{MediaKind, VideoDownloader};

#[test]
fn test_video_sniffer_detects_formats() {
    let downloader = VideoDownloader::new();
    let page_url = "https://example.com/watch";

    // 1. Detect HLS m3u8 playlist
    let hls = downloader.sniff_url("https://cdn.example.com/live/stream.m3u8", page_url, "Live Stream");
    assert!(hls.is_some());
    assert_eq!(hls.unwrap().kind, MediaKind::HlsPlaylist);

    // 2. Detect MPEG-DASH mpd manifest
    let dash = downloader.sniff_url("https://cdn.example.com/vod/manifest.mpd", page_url, "VOD Movie");
    assert!(dash.is_some());
    assert_eq!(dash.unwrap().kind, MediaKind::DashManifest);

    // 3. Detect direct MP4 video
    let mp4 = downloader.sniff_url("https://cdn.example.com/clips/trailer.mp4?token=123", page_url, "Trailer");
    assert!(mp4.is_some());
    assert_eq!(mp4.unwrap().kind, MediaKind::Mp4);

    // 4. Non-media URL should be ignored
    let non_media = downloader.sniff_url("https://cdn.example.com/style.css", page_url, "CSS");
    assert!(non_media.is_none());

    // 5. Streams for page
    let streams = downloader.streams_for_page(page_url);
    assert_eq!(streams.len(), 3);
}
