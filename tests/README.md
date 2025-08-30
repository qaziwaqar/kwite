# Kwite Testing Documentation

This document describes the comprehensive testing suite implemented for the Kwite AI noise cancellation application.

## Test Overview

The testing suite includes:
- **Unit Tests**: Test individual functions and modules in isolation
- **Integration Tests**: Test how modules work together
- **Performance Tests**: Benchmark performance characteristics
- **Cross-Platform Tests**: Test platform-specific behavior
- **Error Handling Tests**: Test graceful error recovery

## Test Structure

### Unit Tests

#### Audio Device Tests (`tests/unit_audio_devices.rs`)
- **Device Info Display**: Test `AudioDeviceInfo` display formatting
- **Device Enumeration**: Test input/output device listing
- **Device Lookup**: Test device retrieval by ID
- **Virtual Device Detection**: Test virtual audio device identification
- **Fallback Behavior**: Test graceful fallback when no devices available
- **ID Uniqueness**: Ensure device IDs are unique
- **Enumeration Consistency**: Verify consistent results across calls

#### Configuration Tests (`tests/unit_config.rs`)
- **Default Configuration**: Test default config values
- **Serialization/Deserialization**: Test TOML round-trip operations
- **Invalid Config Handling**: Test recovery from corrupted config files
- **Unicode Support**: Test config with unicode device names
- **Edge Cases**: Test extreme values and special characters
- **Partial Config**: Test handling of incomplete config files

#### Logger Tests (`tests/unit_logger.rs`)
- **Initialization**: Test logger setup and multiple initialization safety
- **Log Macros**: Test all log levels (debug, info, warn, error)
- **Thread Safety**: Test concurrent logging from multiple threads
- **Performance**: Test logging performance under load
- **Special Characters**: Test logging with unicode and special chars
- **Environment Variables**: Test `RUST_LOG` environment variable handling

### Integration Tests (`tests/integration_tests.rs`)

#### Device-Config Integration
- **Device Selection Workflow**: Test device enumeration → selection → config storage
- **Device Switching**: Test runtime device switching scenarios
- **Fallback Handling**: Test graceful degradation when configured devices missing

#### Application Startup Workflow
- **Config Loading**: Test configuration loading with validation
- **Device Validation**: Test device availability checking
- **Fallback Selection**: Test automatic fallback device selection

#### Virtual Device Workflows
- **Preference Logic**: Test virtual device detection and preference
- **Configuration**: Test config with virtual devices

#### Sensitivity Configuration
- **Precision**: Test floating-point sensitivity value preservation
- **Range Testing**: Test various sensitivity values

### Cross-Platform Tests (`tests/cross_platform_tests.rs`)

#### Platform-Specific Paths
- **Windows**: Test `%APPDATA%\Kwite\config.toml` path expectations
- **macOS**: Test `~/Library/Application Support/Kwite/config.toml` expectations  
- **Linux**: Test `~/.config/kwite/config.toml` expectations

#### Device Naming
- **UTF-8 Validation**: Ensure device names are valid across platforms
- **Null Byte Protection**: Test for cross-platform null byte issues
- **Unicode Handling**: Test unicode device names on all platforms

#### Virtual Device Detection
- **Windows**: VB-Audio, Voicemeeter, Virtual Audio Cable
- **macOS**: BlackHole, Soundflower, Loopback
- **Linux**: PulseAudio, ALSA virtual devices

#### Audio Backend Compatibility
- **Windows**: WASAPI compatibility testing
- **macOS**: Core Audio compatibility testing
- **Linux**: ALSA/PulseAudio compatibility testing

#### Platform-Specific Features
- **Line Endings**: Test Windows (`\r\n`), Unix (`\n`), Mac (`\r`) compatibility
- **Floating Point**: Test cross-platform floating-point precision
- **Path Handling**: Test different path separator handling

### Error Handling Tests (`tests/error_handling_tests.rs`)

#### Missing Device Scenarios
- **Invalid Device IDs**: Test lookup of nonexistent devices
- **Empty Device Lists**: Test behavior with no available devices
- **Special Characters**: Test device IDs with problematic characters

#### Corrupted Configuration
- **Invalid TOML**: Test various TOML syntax errors
- **Binary Data**: Test non-UTF8 config file content
- **Partial Files**: Test incomplete or truncated config files
- **Permission Errors**: Test filesystem permission issues

#### Resource Management
- **Memory Pressure**: Test behavior under simulated memory pressure
- **Concurrent Access**: Test thread-safe device enumeration
- **Resource Leaks**: Test repeated operations for resource leaks
- **Extreme Values**: Test edge cases for sensitivity and other parameters

#### Logging Error Conditions
- **Problematic Strings**: Test logging with null bytes, unicode, long strings
- **Panic Recovery**: Test that logging doesn't cause application crashes

### Performance Tests (`benches/audio_performance.rs`)

#### Device Operations
- **Enumeration Speed**: Benchmark device listing performance
- **Lookup Performance**: Benchmark device ID resolution
- **Virtual Device Search**: Benchmark virtual device detection

#### Configuration Operations
- **Serialization Speed**: Benchmark TOML serialization
- **Deserialization Speed**: Benchmark TOML parsing
- **Round-trip Performance**: Benchmark full save/load cycle

#### Memory and Concurrency
- **Memory Usage**: Test memory efficiency under load
- **Concurrent Access**: Test performance with multiple threads
- **Latency-Critical**: Test real-time operation requirements

#### Edge Case Performance
- **Large Device IDs**: Test performance with long device names
- **Unicode Performance**: Test unicode string handling speed
- **Invalid Lookups**: Test performance of failed device lookups

## Running Tests

### All Tests
```bash
cargo test
```

### Specific Test Categories
```bash
# Unit tests only
cargo test unit_

# Integration tests
cargo test --test integration_tests

# Cross-platform tests
cargo test --test cross_platform_tests

# Error handling tests
cargo test --test error_handling_tests

# Performance benchmarks
cargo bench
```

### Test with Single Thread (for audio device tests)
```bash
cargo test -- --test-threads=1
```

### Verbose Output
```bash
cargo test -- --nocapture
```

## Test Environment Considerations

### CI/CD Environment
- **No Audio Hardware**: Tests gracefully handle environments without real audio devices
- **Fallback Devices**: Tests ensure fallback devices are always available
- **ALSA Errors**: ALSA errors in CI are expected and handled gracefully
- **Timeouts**: Tests have appropriate timeouts for CI environments

### Development Environment  
- **Real Devices**: Tests can detect and work with actual audio hardware
- **Virtual Devices**: Tests can identify virtual audio cables if installed
- **Platform-Specific**: Tests adapt behavior based on the development platform

### Performance Testing
- **Baseline Measurements**: Benchmarks establish performance baselines
- **Regression Detection**: Regular benchmark runs can detect performance regressions
- **Resource Monitoring**: Tests monitor memory usage and thread safety

## Error Handling Philosophy

The testing suite validates that the application follows these error handling principles:

1. **Graceful Degradation**: Always provide working fallbacks
2. **No Panics**: Never crash on invalid input or missing resources
3. **User Feedback**: Provide meaningful error messages where appropriate
4. **Recovery**: Automatically recover from transient errors
5. **Logging**: Log errors appropriately without spamming

## Test Coverage Goals

- **Audio Module**: Test all public device enumeration and lookup functions
- **Config Module**: Test all serialization, path resolution, and error handling
- **Logger Module**: Test initialization, thread safety, and performance
- **Integration**: Test complete workflows end-to-end
- **Error Cases**: Test all identified failure modes
- **Performance**: Establish benchmarks for all critical operations

## Continuous Integration

The test suite is designed to run in CI environments with:
- Automatic dependency installation (ALSA development libraries)
- Graceful handling of missing audio hardware
- Parallel test execution where safe
- Performance regression detection
- Cross-platform compatibility validation

## Contributing to Tests

When adding new features to Kwite:

1. **Add Unit Tests**: Test the new functionality in isolation
2. **Add Integration Tests**: Test how it works with existing features
3. **Consider Error Cases**: Add tests for failure scenarios
4. **Update Performance Tests**: Add benchmarks for performance-critical features
5. **Test Cross-Platform**: Ensure new features work on all supported platforms
6. **Document**: Update this documentation with new test categories

The comprehensive testing suite ensures that Kwite maintains high quality, reliability, and performance across all supported platforms and usage scenarios.