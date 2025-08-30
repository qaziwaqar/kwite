use kwite::logger;
use serial_test::serial;
use std::sync::Once;

static INIT: Once = Once::new();

fn ensure_logger_init() {
    INIT.call_once(|| {
        let _ = logger::init_logger();
    });
}

#[test]
#[serial]
fn test_logger_initialization() {
    // Test that logger initialization doesn't panic
    let result = logger::init_logger();
    assert!(result.is_ok(), "Logger initialization should succeed");
}

#[test]
#[serial]
fn test_logger_multiple_initialization() {
    // Test that multiple calls to init_logger are safe
    ensure_logger_init();
    
    let result1 = logger::init_logger();
    let result2 = logger::init_logger();
    
    assert!(result1.is_ok(), "First logger init should succeed");
    assert!(result2.is_ok(), "Second logger init should not panic");
}

#[test]
#[serial]
fn test_log_macros_compile() {
    ensure_logger_init();
    
    // Test that log macros compile and don't panic
    kwite::logger::log::info!("Test info message");
    kwite::logger::log::warn!("Test warning message");
    kwite::logger::log::error!("Test error message");
    kwite::logger::log::debug!("Test debug message");
}

#[test]
#[serial]
fn test_log_macros_with_formatting() {
    ensure_logger_init();
    
    let device_name = "test_device";
    let count = 42;
    let sensitivity = 0.5;
    
    // Test formatted log messages
    kwite::logger::log::info!("Starting audio processing with device: {}", device_name);
    kwite::logger::log::warn!("Device {} not found, falling back to default", device_name);
    kwite::logger::log::error!("Failed to initialize audio stream: {}", "test error");
    kwite::logger::log::debug!("Processing {} samples with sensitivity {}", count, sensitivity);
}

#[test]
#[serial]
fn test_log_levels() {
    ensure_logger_init();
    
    // Test different log levels work
    kwite::logger::log::error!("Error level message");
    kwite::logger::log::warn!("Warning level message");
    kwite::logger::log::info!("Info level message");
    kwite::logger::log::debug!("Debug level message");
    
    // Test with structured data
    kwite::logger::log::info!(
        device_id = "test_device",
        sample_rate = 44100,
        "Audio device configured"
    );
}

#[test]
#[serial]
fn test_logger_thread_safety() {
    ensure_logger_init();
    
    use std::thread;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicU32, Ordering};
    
    let counter = Arc::new(AtomicU32::new(0));
    let mut handles = vec![];
    
    // Spawn multiple threads that log concurrently
    for i in 0..5 {
        let counter_clone = Arc::clone(&counter);
        let handle = thread::spawn(move || {
            for j in 0..10 {
                kwite::logger::log::info!("Thread {} iteration {}", i, j);
                counter_clone.fetch_add(1, Ordering::SeqCst);
            }
        });
        handles.push(handle);
    }
    
    // Wait for all threads to complete
    for handle in handles {
        handle.join().expect("Thread should complete successfully");
    }
    
    // Verify all log calls were made
    assert_eq!(counter.load(Ordering::SeqCst), 50);
}

#[test]
#[serial]
fn test_log_with_special_characters() {
    ensure_logger_init();
    
    // Test logging with special characters and unicode
    kwite::logger::log::info!("Testing unicode: 音频处理 🎵");
    kwite::logger::log::warn!("Special chars: {}{}{}\"'\\", "<", ">", "&");
    kwite::logger::log::debug!("Newlines and tabs:\n\tTabbed content");
}

#[test]
#[serial]
fn test_log_performance() {
    ensure_logger_init();
    
    use std::time::Instant;
    
    let start = Instant::now();
    
    // Log many messages quickly to test performance
    for i in 0..1000 {
        kwite::logger::log::debug!("Performance test message {}", i);
    }
    
    let duration = start.elapsed();
    
    // This is a smoke test - we just want to ensure logging doesn't
    // take an unreasonably long time
    assert!(duration.as_secs() < 5, "Logging 1000 messages took too long: {:?}", duration);
}

#[cfg(test)]
mod logger_integration_tests {
    use super::*;
    
    #[test]
    #[serial]
    fn test_logger_with_env_var() {
        // Test that logger respects environment variables
        // Note: This is a smoke test as we can't easily change env vars in tests
        std::env::set_var("RUST_LOG", "debug");
        
        let result = logger::init_logger();
        assert!(result.is_ok());
        
        kwite::logger::log::debug!("This debug message should be visible");
        
        // Clean up
        std::env::remove_var("RUST_LOG");
    }
}