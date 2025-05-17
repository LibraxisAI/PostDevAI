// Main test file for including all test modules
// Add all test modules here so they are included in cargo test

// TUI tests
#[cfg(test)]
mod tui {
    pub mod system_bridge_test;
    pub mod app_state_test;
    pub mod view_helper_test;
}

// Make sure the TUI function exports work
#[test]
fn test_tui_exports() {
    // Test importing and using various TUI components
    // This verifies they are properly exported and available
    
    // Import key components
    use postdevai::tui::app::App;
    use postdevai::tui::bridge::SystemBridge;
    use postdevai::tui::state::app_state::AppState;
    use postdevai::tui::views::dashboard::render_dashboard;
    use postdevai::tui::views::models::render_models;
    use postdevai::tui::views::ramlake::render_ramlake;
    use postdevai::tui::views::history::render_history;
    use postdevai::tui::views::context::render_context;
    use postdevai::tui::views::help::render_help;
    
    // Create instances of key components
    let _app = App::new(std::time::Duration::from_millis(250));
    let _bridge = SystemBridge::new();
    let _state = AppState::new();
    
    // We can't easily test the render functions without a Frame
    // But we can verify they're imported correctly
    
    // Verify exports are working
    assert!(true);
}