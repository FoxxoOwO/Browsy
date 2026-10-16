use browsy::adblock::AdblockEngine;

#[test]
fn test_adblock_engine_filters_ads() {
    let engine = AdblockEngine::new();

    // 1. Should block known ad networks from third party
    let ad_decision = engine.check_request(
        "https://googleadservices.com/pagead/conversion.js",
        "https://news.ycombinator.com/",
        "script",
    );
    assert!(ad_decision.should_block, "googleadservices should be blocked");

    let pop_decision = engine.check_request(
        "https://popads.net/serve.js",
        "https://example.com/",
        "script",
    );
    assert!(pop_decision.should_block, "popads should be blocked");

    // 2. Should allow clean first-party resources
    let clean_decision = engine.check_request(
        "https://news.ycombinator.com/hn.js",
        "https://news.ycombinator.com/",
        "script",
    );
    assert!(!clean_decision.should_block, "clean script should be allowed");
}

#[test]
fn test_adblock_allowlist_bypass() {
    let engine = AdblockEngine::new();
    let domain = "allowed-site.com";

    // Add to allowlist
    assert!(engine.toggle_allowlist(domain));
    assert!(engine.is_allowlisted(domain));

    // When site is allowlisted, requests originating from it are not blocked
    let decision = engine.check_request(
        "https://googleadservices.com/pagead/conversion.js",
        "https://allowed-site.com/article",
        "script",
    );
    assert!(!decision.should_block, "allowlisted site should bypass filters");

    // Toggle back off
    assert!(!engine.toggle_allowlist(domain));
    assert!(!engine.is_allowlisted(domain));
}

#[test]
fn test_adblock_cosmetic_filtering() {
    let engine = AdblockEngine::new();
    let css = engine.get_cosmetic_css("https://example.com/");
    // Should produce display: none !important rules
    if !css.is_empty() {
        assert!(css.contains("display: none !important;"));
    }
}
