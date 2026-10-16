use browsy::extensions::manifest::ExtensionManifest;
use browsy::extensions::matcher::PatternMatcher;
use browsy::extensions::polyfill::generate_webextensions_polyfill;

#[test]
fn test_manifest_v2_and_v3_parsing() {
    let mv3_json = r#"{
        "manifest_version": 3,
        "name": "Dark Mode Extension",
        "version": "1.2.3",
        "description": "Inverts colors for night reading",
        "permissions": ["storage", "tabs"],
        "content_scripts": [
            {
                "matches": ["*://*.wikipedia.org/*", "https://news.ycombinator.com/*"],
                "js": ["dark.js"],
                "css": ["dark.css"]
            }
        ],
        "action": {
            "default_title": "Toggle Dark Mode",
            "default_popup": "popup.html"
        }
    }"#;

    let manifest = ExtensionManifest::parse_str(mv3_json).expect("MV3 parse failed");
    assert_eq!(manifest.manifest_version, 3);
    assert_eq!(manifest.name, "Dark Mode Extension");
    assert_eq!(manifest.version, "1.2.3");
    assert_eq!(manifest.permissions.len(), 2);
    assert_eq!(manifest.get_popup_file(), Some("popup.html".to_string()));
}

#[test]
fn test_pattern_matcher() {
    let matcher = PatternMatcher::new("*://*.example.com/*").unwrap();
    assert!(matcher.matches("https://sub.example.com/test"));
    assert!(matcher.matches("http://example.com/"));
    assert!(!matcher.matches("https://other.org/test"));

    let all_urls = PatternMatcher::new("<all_urls>").unwrap();
    assert!(all_urls.matches("https://google.com/"));
    assert!(all_urls.matches("http://localhost:8080/app"));
}

#[test]
fn test_webextensions_polyfill_generation() {
    let polyfill = generate_webextensions_polyfill("dark_reader", "Dark Reader");
    assert!(polyfill.contains("window.chrome"));
    assert!(polyfill.contains("window.browser"));
    assert!(polyfill.contains("sendMessage"));
    assert!(polyfill.contains("storageStore"));
}
