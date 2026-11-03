//! Application: use case/service yang hanya bergantung pada `domain`.
pub mod bridge_service;
pub mod charger_service;
pub mod clipboard_service;
pub mod health_service;
pub mod island_service;
pub mod media_service;
pub mod pomodoro_service;
pub mod quick_ask_service;
pub mod review;
pub mod settings_service;
#[cfg(test)]
pub mod test_support;
