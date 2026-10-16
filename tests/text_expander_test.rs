use browsy::text_expander::TextExpander;

#[test]
fn test_text_expander_shortcuts() {
    let expander = TextExpander::new();

    // 1. :email trigger
    let input = "Send email to :email";
    let expanded = expander.try_expand_input(input);
    assert!(expanded.is_some());
    assert_eq!(expanded.unwrap(), "Send email to user@example.com");

    // 2. :shrug trigger
    let input_shrug = "I don't know :shrug";
    let expanded_shrug = expander.try_expand_input(input_shrug);
    assert!(expanded_shrug.is_some());
    assert_eq!(expanded_shrug.unwrap(), "I don't know ¯\\_(ツ)_/¯");

    // 3. Dynamic date trigger
    let input_date = "Today is :date";
    let expanded_date = expander.try_expand_input(input_date);
    assert!(expanded_date.is_some());
    let res = expanded_date.unwrap();
    assert!(res.starts_with("Today is 20")); // 202x-xx-xx

    // 4. Custom snippet
    let custom_id = expander.add_snippet(":sig", "Cheers,\nDev", "Sign-off");
    let input_custom = "Thanks, :sig";
    let expanded_custom = expander.try_expand_input(input_custom);
    assert_eq!(expanded_custom.unwrap(), "Thanks, Cheers,\nDev");

    // 5. Disable snippet
    expander.toggle_enabled(custom_id);
    let after_disable = expander.try_expand_input(input_custom);
    assert!(after_disable.is_none());
}

#[test]
fn test_text_expander_script_generation() {
    let expander = TextExpander::new();
    let script = expander.generate_content_script();
    assert!(script.contains(":email"));
    assert!(script.contains("addEventListener('input'"));
}
