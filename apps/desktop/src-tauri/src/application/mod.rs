//! Application: use case/service yang hanya bergantung pada `domain`.
pub mod health_service;
pub mod island_service;
pub mod media_service;
pub mod review;
pub mod settings_service;
#[cfg(test)]
pub mod test_support;
