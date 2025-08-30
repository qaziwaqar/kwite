use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use kwite::audio::devices::{list_input_devices, list_output_devices, get_device_by_id, find_virtual_output_device};
use kwite::config::KwiteConfig;
use kwite::logger;
use std::time::Duration;

fn benchmark_device_enumeration(c: &mut Criterion) {
    let _ = logger::init_logger();
    
    let mut group = c.benchmark_group("device_enumeration");
    
    group.bench_function("list_input_devices", |b| {
        b.iter(|| black_box(list_input_devices()))
    });
    
    group.bench_function("list_output_devices", |b| {
        b.iter(|| black_box(list_output_devices()))
    });
    
    group.bench_function("both_device_lists", |b| {
        b.iter(|| {
            let input = black_box(list_input_devices());
            let output = black_box(list_output_devices());
            (input, output)
        })
    });
    
    group.finish();
}

fn benchmark_device_lookup(c: &mut Criterion) {
    let _ = logger::init_logger();
    
    let input_devices = list_input_devices();
    let output_devices = list_output_devices();
    
    if input_devices.is_empty() || output_devices.is_empty() {
        return; // Skip if no devices available
    }
    
    let mut group = c.benchmark_group("device_lookup");
    
    let input_id = &input_devices[0].id;
    let output_id = &output_devices[0].id;
    
    group.bench_with_input(
        BenchmarkId::new("get_input_device_by_id", input_id),
        input_id,
        |b, id| {
            b.iter(|| black_box(get_device_by_id(id, true)))
        }
    );
    
    group.bench_with_input(
        BenchmarkId::new("get_output_device_by_id", output_id),
        output_id,
        |b, id| {
            b.iter(|| black_box(get_device_by_id(id, false)))
        }
    );
    
    group.bench_function("find_virtual_output_device", |b| {
        b.iter(|| black_box(find_virtual_output_device()))
    });
    
    // Benchmark invalid device lookup
    group.bench_function("invalid_device_lookup", |b| {
        b.iter(|| {
            let input_result = black_box(get_device_by_id("nonexistent_input", true));
            let output_result = black_box(get_device_by_id("nonexistent_output", false));
            (input_result, output_result)
        })
    });
    
    group.finish();
}

fn benchmark_config_operations(c: &mut Criterion) {
    let _ = logger::init_logger();
    
    let mut group = c.benchmark_group("config_operations");
    
    group.bench_function("config_default", |b| {
        b.iter(|| black_box(KwiteConfig::default()))
    });
    
    group.bench_function("config_load", |b| {
        b.iter(|| black_box(KwiteConfig::load()))
    });
    
    let config = KwiteConfig::default();
    
    group.bench_function("config_serialize", |b| {
        b.iter(|| black_box(toml::to_string_pretty(&config).unwrap()))
    });
    
    let toml_content = toml::to_string_pretty(&config).unwrap();
    
    group.bench_function("config_deserialize", |b| {
        b.iter(|| black_box(toml::from_str::<KwiteConfig>(&toml_content).unwrap()))
    });
    
    group.bench_function("config_roundtrip", |b| {
        b.iter(|| {
            let serialized = black_box(toml::to_string_pretty(&config).unwrap());
            black_box(toml::from_str::<KwiteConfig>(&serialized).unwrap())
        })
    });
    
    group.finish();
}

fn benchmark_memory_usage(c: &mut Criterion) {
    let _ = logger::init_logger();
    
    let mut group = c.benchmark_group("memory_usage");
    
    // Benchmark creating many device lists
    group.bench_function("many_device_enumerations", |b| {
        b.iter(|| {
            for _ in 0..100 {
                let _input = black_box(list_input_devices());
                let _output = black_box(list_output_devices());
            }
        })
    });
    
    // Benchmark creating many configs
    group.bench_function("many_config_creations", |b| {
        b.iter(|| {
            for i in 0..1000 {
                let _config = black_box(KwiteConfig {
                    input_device_id: format!("input_{}", i),
                    output_device_id: format!("output_{}", i),
                    sensitivity: (i as f32) / 1000.0,
                    auto_start: i % 2 == 0,
                    minimize_to_tray: i % 3 == 0,
                    development_mode: false,
                    remote_logging: kwite::remote_logging::RemoteLoggingConfig::default(),
                    analytics: kwite::config::AnalyticsConfig::default(),
                    auto_update: kwite::config::AutoUpdateConfig::default(),
                });
            }
        })
    });
    
    group.finish();
}

fn benchmark_concurrent_access(c: &mut Criterion) {
    let _ = logger::init_logger();
    
    let mut group = c.benchmark_group("concurrent_access");
    
    // Benchmark parallel device enumeration
    group.bench_function("parallel_device_enumeration", |b| {
        b.iter(|| {
            use std::thread;
            let handles: Vec<_> = (0..4).map(|_| {
                thread::spawn(|| {
                    let _input = black_box(list_input_devices());
                    let _output = black_box(list_output_devices());
                })
            }).collect();
            
            for handle in handles {
                handle.join().unwrap();
            }
        })
    });
    
    group.finish();
}

fn benchmark_latency_critical_operations(c: &mut Criterion) {
    let _ = logger::init_logger();
    
    let mut group = c.benchmark_group("latency_critical");
    
    // These operations should be very fast for real-time audio
    group.measurement_time(Duration::from_secs(10));
    group.sample_size(1000);
    
    let devices = list_output_devices();
    if !devices.is_empty() {
        let device_id = &devices[0].id;
        
        group.bench_function("fast_device_lookup", |b| {
            b.iter(|| black_box(get_device_by_id(device_id, false)))
        });
    }
    
    let config = KwiteConfig::default();
    group.bench_function("config_access", |b| {
        b.iter(|| {
            black_box(&config.sensitivity);
            black_box(&config.input_device_id);
            black_box(&config.output_device_id);
        })
    });
    
    group.finish();
}

fn benchmark_edge_cases(c: &mut Criterion) {
    let _ = logger::init_logger();
    
    let mut group = c.benchmark_group("edge_cases");
    
    // Benchmark with various config sizes
    for &size in &[1, 10, 100, 1000] {
        group.bench_with_input(
            BenchmarkId::new("large_device_id", size),
            &size,
            |b, &size| {
                let large_id = "device_".repeat(size);
                b.iter(|| black_box(get_device_by_id(&large_id, true)))
            }
        );
    }
    
    // Benchmark unicode device names
    let unicode_configs = vec![
        ("ascii", "normal_device"),
        ("unicode", "设备_麦克风_🎵"),
        ("mixed", "device_麦克风_123"),
    ];
    
    for (name, device_id) in unicode_configs {
        group.bench_with_input(
            BenchmarkId::new("unicode_lookup", name),
            &device_id,
            |b, id| {
                b.iter(|| black_box(get_device_by_id(id, false)))
            }
        );
    }
    
    group.finish();
}

criterion_group!(
    device_benches,
    benchmark_device_enumeration,
    benchmark_device_lookup,
    benchmark_latency_critical_operations
);

criterion_group!(
    config_benches,
    benchmark_config_operations
);

criterion_group!(
    stress_benches,
    benchmark_memory_usage,
    benchmark_concurrent_access,
    benchmark_edge_cases
);

criterion_main!(device_benches, config_benches, stress_benches);