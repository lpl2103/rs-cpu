//! CPU Benchmark Engine supporting Single-Thread, Multi-Thread, and Stress tests.

use rayon::prelude::*;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// Benchmark reference CPU profile for direct score comparisons.
#[derive(Debug, Clone)]
pub struct BenchmarkReference {
    /// Processor description.
    pub name: &'static str,
    /// Single thread reference score.
    pub single_score: f64,
    /// Multi thread reference score.
    pub multi_score: f64,
    /// Physical core and thread count string.
    pub config: &'static str,
}

/// Standard reference benchmark database.
pub const REFERENCE_CPUS: &[BenchmarkReference] = &[
    BenchmarkReference {
        name: "Intel Core i9-14900K",
        single_score: 915.0,
        multi_score: 17120.0,
        config: "24C / 32T (8P + 16E)",
    },
    BenchmarkReference {
        name: "AMD Ryzen 9 7950X",
        single_score: 785.0,
        multi_score: 15650.0,
        config: "16C / 32T",
    },
    BenchmarkReference {
        name: "Intel Core i7-14700K",
        single_score: 890.0,
        multi_score: 14850.0,
        config: "20C / 28T (8P + 12E)",
    },
    BenchmarkReference {
        name: "AMD Ryzen 7 7800X3D",
        single_score: 715.0,
        multi_score: 7650.0,
        config: "8C / 16T (3D V-Cache)",
    },
    BenchmarkReference {
        name: "Intel Core i5-14600K",
        single_score: 840.0,
        multi_score: 10200.0,
        config: "14C / 20T (6P + 8E)",
    },
    BenchmarkReference {
        name: "AMD Ryzen 5 7600X",
        single_score: 755.0,
        multi_score: 5950.0,
        config: "6C / 12T",
    },
    BenchmarkReference {
        name: "AMD Ryzen 5 5600X",
        single_score: 620.0,
        multi_score: 4850.0,
        config: "6C / 12T (Zen 3)",
    },
    BenchmarkReference {
        name: "Intel Core i7-8700K",
        single_score: 550.0,
        multi_score: 3750.0,
        config: "6C / 12T (Coffee Lake)",
    },
];

/// Current state of the benchmark execution.
#[derive(Debug, Clone, PartialEq)]
pub enum BenchStatus {
    /// Benchmark engine idle.
    Idle,
    /// Executing single-thread workload.
    RunningSingle {
        /// Current progress fraction from 0.0 to 1.0.
        progress: f32,
    },
    /// Executing multi-thread workload.
    RunningMulti {
        /// Current progress fraction from 0.0 to 1.0.
        progress: f32,
    },
    /// Stress test actively running.
    StressTesting {
        /// Number of elapsed seconds since the stress test started.
        elapsed_secs: u64,
    },
    /// Benchmark completed with calculated scores.
    Completed,
}

/// Benchmark controller and result holder.
#[derive(Clone)]
pub struct BenchManager {
    /// Calculated single-thread score.
    pub single_score: Arc<Mutex<f64>>,
    /// Calculated multi-thread score.
    pub multi_score: Arc<Mutex<f64>>,
    /// Status of current execution.
    pub status: Arc<Mutex<BenchStatus>>,
    /// Cancellation flag for stopping stress tests or running benchmarks.
    pub cancel_flag: Arc<AtomicBool>,
    /// Selected index of reference CPU for UI comparison.
    pub selected_ref_idx: usize,
}

impl Default for BenchManager {
    fn default() -> Self {
        Self {
            single_score: Arc::new(Mutex::new(0.0)),
            multi_score: Arc::new(Mutex::new(0.0)),
            status: Arc::new(Mutex::new(BenchStatus::Idle)),
            cancel_flag: Arc::new(AtomicBool::new(false)),
            selected_ref_idx: 0,
        }
    }
}

impl BenchManager {
    /// Starts the complete benchmark sequence (Single Thread -> Multi Thread).
    pub fn start_bench(&self, thread_count: usize) {
        let single_score = Arc::clone(&self.single_score);
        let multi_score = Arc::clone(&self.multi_score);
        let status = Arc::clone(&self.status);
        let cancel = Arc::clone(&self.cancel_flag);

        cancel.store(false, Ordering::SeqCst);

        std::thread::spawn(move || {
            // 1. Single Thread Benchmark
            if let Ok(mut s) = status.lock() {
                *s = BenchStatus::RunningSingle { progress: 0.0 };
            }

            let start = Instant::now();
            let duration = Duration::from_millis(2000);
            let mut iterations: u64 = 0;

            while start.elapsed() < duration {
                if cancel.load(Ordering::Relaxed) {
                    if let Ok(mut s) = status.lock() {
                        *s = BenchStatus::Idle;
                    }
                    return;
                }
                benchmark_worker_chunk(25_000);
                iterations += 25_000;

                let progress = (start.elapsed().as_secs_f32() / duration.as_secs_f32()).min(1.0);
                if let Ok(mut s) = status.lock() {
                    *s = BenchStatus::RunningSingle { progress };
                }
            }

            let single_calculated = (iterations as f64) / 12_500.0;
            if let Ok(mut sc) = single_score.lock() {
                *sc = single_calculated.round();
            }

            // Brief pause between runs
            std::thread::sleep(Duration::from_millis(200));

            // 2. Multi Thread Benchmark
            if let Ok(mut s) = status.lock() {
                *s = BenchStatus::RunningMulti { progress: 0.0 };
            }

            let start_multi = Instant::now();
            let multi_duration = Duration::from_millis(2500);
            let total_multi_iters = Arc::new(std::sync::atomic::AtomicU64::new(0));

            let pool = rayon::ThreadPoolBuilder::new()
                .num_threads(thread_count)
                .build();

            if let Ok(p) = pool {
                while start_multi.elapsed() < multi_duration {
                    if cancel.load(Ordering::Relaxed) {
                        if let Ok(mut s) = status.lock() {
                            *s = BenchStatus::Idle;
                        }
                        return;
                    }

                    let iters_ref = Arc::clone(&total_multi_iters);
                    p.install(|| {
                        (0..thread_count).into_par_iter().for_each(|_| {
                            benchmark_worker_chunk(25_000);
                            iters_ref.fetch_add(25_000, Ordering::Relaxed);
                        });
                    });

                    let progress = (start_multi.elapsed().as_secs_f32() / multi_duration.as_secs_f32()).min(1.0);
                    if let Ok(mut s) = status.lock() {
                        *s = BenchStatus::RunningMulti { progress };
                    }
                }
            }

            let multi_calculated = (total_multi_iters.load(Ordering::SeqCst) as f64) / 15_000.0;
            if let Ok(mut mc) = multi_score.lock() {
                *mc = multi_calculated.round();
            }

            if let Ok(mut s) = status.lock() {
                *s = BenchStatus::Completed;
            }
        });
    }

    /// Toggles CPU Stress Test.
    pub fn toggle_stress(&self, thread_count: usize) {
        let is_running = self
            .status
            .lock()
            .is_ok_and(|s| matches!(*s, BenchStatus::StressTesting { .. }));

        if is_running {
            self.cancel_flag.store(true, Ordering::SeqCst);
            if let Ok(mut s) = self.status.lock() {
                *s = BenchStatus::Idle;
            }
        } else {
            let status = Arc::clone(&self.status);
            let cancel = Arc::clone(&self.cancel_flag);
            cancel.store(false, Ordering::SeqCst);

            std::thread::spawn(move || {
                let start = Instant::now();
                let pool = rayon::ThreadPoolBuilder::new()
                    .num_threads(thread_count)
                    .build();

                if let Ok(p) = pool {
                    while !cancel.load(Ordering::Relaxed) {
                        p.install(|| {
                            (0..thread_count).into_par_iter().for_each(|_| {
                                benchmark_worker_chunk(40_000);
                            });
                        });

                        if let Ok(mut s) = status.lock() {
                            *s = BenchStatus::StressTesting {
                                elapsed_secs: start.elapsed().as_secs(),
                            };
                        }
                    }
                }

                if let Ok(mut s) = status.lock() {
                    *s = BenchStatus::Idle;
                }
            });
        }
    }

    /// Stops any running bench or stress test.
    pub fn stop(&self) {
        self.cancel_flag.store(true, Ordering::SeqCst);
    }
}

/// Deterministic mathematical workload (Mandelbrot arithmetic & Bitwise transformation).
fn benchmark_worker_chunk(iterations: usize) {
    let mut acc: f64 = 0.5;
    for i in 0..iterations {
        let x = (i as f64) * 0.001;
        acc = (acc * x + 0.314_159_265_358_979_3).sin().cos().abs();
        let _ = std::hint::black_box(acc);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_benchmark_chunk_runs() {
        benchmark_worker_chunk(100);
    }

    #[test]
    fn test_benchmark_references_not_empty() {
        assert!(!REFERENCE_CPUS.is_empty());
        assert!(REFERENCE_CPUS[0].single_score > 0.0);
        assert!(REFERENCE_CPUS[0].multi_score > 0.0);
    }

    #[test]
    fn test_bench_manager_default() {
        let mgr = BenchManager::default();
        let s = *mgr.single_score.lock().unwrap();
        let m = *mgr.multi_score.lock().unwrap();
        assert!(s.abs() < f64::EPSILON);
        assert!(m.abs() < f64::EPSILON);
    }
}
