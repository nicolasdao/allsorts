// Integration test that includes the specs/250906_01_bug test files

#[path = "../specs/250906_01_bug/allsorts_integration.rs"]
mod allsorts_integration;

#[path = "../specs/250906_01_bug/test.rs"]
mod test;

// Re-export the tests so they can be run
pub use test::*;