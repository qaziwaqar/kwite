//! Benchmarks for AI noise cancellation performance
//! 
//! These benchmarks measure the AI processing performance to ensure Kwite
//! meets professional-grade standards comparable to Krisp.ai

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use kwite::audio::process::process_audio;
use kwite::ai_metrics::{AiMetrics, create_shared_metrics};
use nnnoiseless::DenoiseState;
use std::time::Duration;

fn create_test_denoiser() -> DenoiseState<'static> {
    unsafe {
        std::mem::transmute::<DenoiseState<'_>, DenoiseState<'static>>(*DenoiseState::new())
    }
}

fn benchmark_ai_processing_latency(c: &mut Criterion) {
    let mut group = c.benchmark_group("ai_processing_latency");
    
    // Test different frame sizes to understand AI performance characteristics
    let frame_sizes = vec![480, 960, 1440]; // 1, 2, 3 frames
    
    for &size in &frame_sizes {
        group.bench_with_input(
            BenchmarkId::new("ai_process_frame", size),
            &size,
            |b, &size| {
                let mut denoiser = create_test_denoiser();
                let input = vec![0.1; size];
                let mut output = vec![0.0; size];
                
                b.iter(|| {
                    process_audio(
                        black_box(&input),
                        black_box(&mut output),
                        black_box(&mut denoiser),
                        None
                    );
                });
            },
        );
    }
    
    group.finish();
}

fn benchmark_ai_processing_with_metrics(c: &mut Criterion) {
    let mut group = c.benchmark_group("ai_processing_with_metrics");
    
    group.bench_function("ai_process_with_metrics", |b| {
        let mut denoiser = create_test_denoiser();
        let metrics = create_shared_metrics();
        let input = vec![0.1; 480];
        let mut output = vec![0.0; 480];
        
        b.iter(|| {
            process_audio(
                black_box(&input),
                black_box(&mut output),
                black_box(&mut denoiser),
                Some(black_box(&metrics))
            );
        });
    });
    
    group.bench_function("ai_process_without_metrics", |b| {
        let mut denoiser = create_test_denoiser();
        let input = vec![0.1; 480];
        let mut output = vec![0.0; 480];
        
        b.iter(|| {
            process_audio(
                black_box(&input),
                black_box(&mut output),
                black_box(&mut denoiser),
                None
            );
        });
    });
    
    group.finish();
}

fn benchmark_ai_metrics_performance(c: &mut Criterion) {
    let mut group = c.benchmark_group("ai_metrics_performance");
    
    group.bench_function("metrics_record_frame", |b| {
        let mut metrics = AiMetrics::new();
        
        b.iter(|| {
            metrics.record_frame(
                black_box(0.75),
                black_box(Duration::from_micros(5000))
            );
        });
    });
    
    group.bench_function("metrics_get_summary", |b| {
        let mut metrics = AiMetrics::new();
        
        // Pre-populate with data
        for i in 0..100 {
            metrics.record_frame(0.7, Duration::from_micros(5000));
        }
        
        b.iter(|| {
            black_box(metrics.get_performance_summary());
        });
    });
    
    group.finish();
}

fn benchmark_real_time_performance(c: &mut Criterion) {
    let mut group = c.benchmark_group("real_time_performance");
    
    // Simulate real-time audio processing at 48kHz
    group.bench_function("real_time_simulation", |b| {
        let mut denoiser = create_test_denoiser();
        let metrics = create_shared_metrics();
        
        // 10ms worth of audio at 48kHz (480 samples)
        let input = vec![0.1; 480];
        let mut output = vec![0.0; 480];
        
        b.iter(|| {
            // This should complete in under 10ms for real-time operation
            process_audio(
                black_box(&input),
                black_box(&mut output),
                black_box(&mut denoiser),
                Some(black_box(&metrics))
            );
        });
    });
    
    // Test sustained processing (100 frames)
    group.bench_function("sustained_processing", |b| {
        let mut denoiser = create_test_denoiser();
        let metrics = create_shared_metrics();
        let input = vec![0.1; 480];
        
        b.iter(|| {
            for _ in 0..100 {
                let mut output = vec![0.0; 480];
                process_audio(
                    black_box(&input),
                    black_box(&mut output),
                    black_box(&mut denoiser),
                    Some(black_box(&metrics))
                );
            }
        });
    });
    
    group.finish();
}

fn benchmark_competitive_performance(c: &mut Criterion) {
    let mut group = c.benchmark_group("competitive_benchmarks");
    group.measurement_time(Duration::from_secs(20)); // Longer measurement for accuracy
    
    // Professional-grade latency test (should be under 5ms per frame)
    group.bench_function("professional_latency", |b| {
        let mut denoiser = create_test_denoiser();
        let input = vec![0.1; 480];
        let mut output = vec![0.0; 480];
        
        b.iter(|| {
            let start = std::time::Instant::now();
            process_audio(
                black_box(&input),
                black_box(&mut output),
                black_box(&mut denoiser),
                None
            );
            let duration = start.elapsed();
            
            // Assert professional-grade performance
            assert!(duration.as_millis() < 5, 
                   "Professional AI processing should be under 5ms per frame");
            
            black_box(duration);
        });
    });
    
    // Throughput test (frames per second)
    group.bench_function("ai_throughput", |b| {
        let mut denoiser = create_test_denoiser();
        let input = vec![0.1; 480];
        
        b.iter(|| {
            let start = std::time::Instant::now();
            let mut frames_processed = 0;
            
            // Process for 100ms
            while start.elapsed().as_millis() < 100 {
                let mut output = vec![0.0; 480];
                process_audio(
                    black_box(&input),
                    black_box(&mut output),
                    black_box(&mut denoiser),
                    None
                );
                frames_processed += 1;
            }
            
            // Should process at least 100 frames in 100ms (1000 fps)
            // for professional real-time performance
            black_box(frames_processed);
        });
    });
    
    group.finish();
}

fn benchmark_vad_accuracy(c: &mut Criterion) {
    let mut group = c.benchmark_group("vad_performance");
    
    // Test VAD score consistency and performance
    group.bench_function("vad_consistency", |b| {
        let mut denoiser = create_test_denoiser();
        let metrics = create_shared_metrics();
        
        // Test with consistent input (should produce consistent VAD)
        let input = vec![0.5; 480]; // Mid-range consistent signal
        let mut output = vec![0.0; 480];
        
        b.iter(|| {
            for _ in 0..10 {
                process_audio(
                    black_box(&input),
                    black_box(&mut output),
                    black_box(&mut denoiser),
                    Some(black_box(&metrics))
                );
            }
            
            // Check VAD consistency
            if let Ok(metrics_guard) = metrics.lock() {
                let summary = metrics_guard.get_performance_summary();
                black_box(summary.model_confidence);
            }
        });
    });
    
    group.finish();
}

criterion_group!(
    ai_benchmarks,
    benchmark_ai_processing_latency,
    benchmark_ai_processing_with_metrics,
    benchmark_ai_metrics_performance,
    benchmark_real_time_performance,
    benchmark_competitive_performance,
    benchmark_vad_accuracy
);

criterion_main!(ai_benchmarks);