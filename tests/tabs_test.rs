use browsy::tabs::{TabLayout, TabManager};

#[test]
fn test_tabs_management() {
    let mut manager = TabManager::new();
    assert_eq!(manager.tabs.len(), 1);

    // New Tab
    let tab2 = manager.new_tab("https://servo.org");
    assert_eq!(manager.tabs.len(), 2);
    assert_eq!(manager.active_tab_id, tab2);

    // Navigation and history
    if let Some(tab) = manager.active_tab_mut() {
        tab.navigate("https://doc.rust-lang.org");
        assert_eq!(tab.url, "https://doc.rust-lang.org");
        assert!(tab.can_go_back);

        let back_url = tab.go_back();
        assert_eq!(back_url, Some("https://servo.org"));
        assert!(tab.can_go_forward);

        let fwd_url = tab.go_forward();
        assert_eq!(fwd_url, Some("https://doc.rust-lang.org"));
    }

    // Toggle Horizontal / Vertical layout
    assert_eq!(manager.layout, TabLayout::Horizontal);
    manager.toggle_layout();
    assert_eq!(manager.layout, TabLayout::Vertical);
    manager.toggle_layout();
    assert_eq!(manager.layout, TabLayout::Horizontal);

    // Close Tab
    manager.close_tab(tab2);
    assert_eq!(manager.tabs.len(), 1);
}
