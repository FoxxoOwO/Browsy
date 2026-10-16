use browsy::popup::PopupBlocker;

#[test]
fn test_popup_blocker_unsolicited_blocked() {
    let blocker = PopupBlocker::new();
    let tab_id = 42;

    // Unsolicited popup (no user interaction recorded)
    let allowed = blocker.evaluate_popup_request(tab_id, "https://sketchy-site.com/page", "https://spam.com/popup");
    assert!(!allowed, "Unsolicited popup must be blocked");

    // Notification must be queued
    let notif = blocker.get_pending_notification(tab_id);
    assert!(notif.is_some(), "Pending notification should exist for tab");
    let n = notif.unwrap();
    assert_eq!(n.source_domain, "sketchy-site.com");

    // Total blocked count incremented
    assert_eq!(blocker.total_blocked(), 1);

    // Dismissing notification
    blocker.dismiss_notification(tab_id);
    assert!(blocker.get_pending_notification(tab_id).is_none());
}

#[test]
fn test_popup_blocker_user_activation_allowed() {
    let blocker = PopupBlocker::new();
    let tab_id = 99;

    // Simulate user click/interaction
    blocker.record_user_interaction(tab_id);

    // Immediate popup request right after user click
    let allowed = blocker.evaluate_popup_request(tab_id, "https://myapp.com/dashboard", "https://oauth.provider.com/auth");
    assert!(allowed, "Popup triggered by user click should be allowed");
}

#[test]
fn test_popup_blocker_allowlist() {
    let blocker = PopupBlocker::new();
    let tab_id = 10;

    // Google accounts is on default allowlist
    let allowed = blocker.evaluate_popup_request(tab_id, "https://accounts.google.com/signin", "https://accounts.google.com/oauth");
    assert!(allowed, "Accounts on allowlist should always be allowed");
}
