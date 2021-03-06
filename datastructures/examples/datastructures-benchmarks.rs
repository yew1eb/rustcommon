// Copyright 2019-2020 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use rustcommon_datastructures::*;
use std::sync::Arc;
use std::{thread, time};

pub const NS_PER_SEC: u64 = 1_000_000_000;
pub const NS_PER_MINUTE: u64 = 60 * NS_PER_SEC;

#[derive(Debug, Copy, Clone)]
pub enum Structure {
    Counter,
    Histogram,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Operation {
    Increment,
    Percentile,
}

pub fn main() {
    let runtime = 5.0;

    runner(
        runtime,
        Structure::Counter,
        Operation::Increment,
        "Counter Incr/s".to_string(),
    );
    runner(
        runtime,
        Structure::Histogram,
        Operation::Increment,
        "Histogram Incr/s".to_string(),
    );
    runner(
        runtime,
        Structure::Histogram,
        Operation::Percentile,
        "Histogram Percentile/s".to_string(),
    );
}

pub fn runner(runtime: f64, structure: Structure, operation: Operation, label: String) {
    match operation {
        Operation::Increment => {
            for single_channel in [true, false].iter() {
                for i in [1, 2, 4, 8, 16, 32, 64].iter() {
                    timed_run(
                        *i,
                        runtime,
                        structure,
                        operation,
                        *single_channel,
                        format!("{} (threads: {})", label, i),
                    );
                }
            }
        }
        Operation::Percentile => {
            for i in [1, 2, 4, 8, 16, 32, 64].iter() {
                timed_run(
                    *i,
                    runtime,
                    structure,
                    operation,
                    false,
                    format!("{} (threads: {})", label, i),
                );
            }
        }
    }
}

pub fn timed_run(
    threads: usize,
    runtime: f64,
    structure: Structure,
    operation: Operation,
    single_channel: bool,
    label: String,
) {
    let max = 100_000;
    let duration = sized_run(threads, max, structure, operation, single_channel);
    let rate = max as f64 / duration;
    let max = (runtime * rate) as usize;
    let duration = sized_run(threads, max, structure, operation, single_channel);
    let rate = max as f64 / duration;
    println!(
        "{} (contended: {}): {:.2e} ops",
        label, single_channel, rate
    );
}

pub fn sized_run(
    threads: usize,
    max: usize,
    structure: Structure,
    operation: Operation,
    contended: bool,
) -> f64 {
    let mut thread_pool = Vec::new();
    let t0 = time::Instant::now();
    match structure {
        Structure::Counter => {
            if contended {
                let counter = Arc::new(AtomicU64::default());
                for _ in 0..threads {
                    let counter = counter.clone();
                    match operation {
                        Operation::Increment => {
                            thread_pool.push(thread::spawn(move || {
                                for _ in 0..(max / threads) {
                                    counter.fetch_add(1, Ordering::Relaxed);
                                }
                            }));
                        }
                        _ => unimplemented!(),
                    }
                }
            } else {
                for _ in 0..threads {
                    let counter = AtomicU64::default();
                    match operation {
                        Operation::Increment => {
                            thread_pool.push(thread::spawn(move || {
                                for _ in 0..(max / threads) {
                                    counter.fetch_add(1, Ordering::Relaxed);
                                }
                            }));
                        }
                        _ => unimplemented!(),
                    }
                }
            }
        }
        Structure::Histogram => {
            let histogram = Arc::new(Histogram::<AtomicU64>::new(NS_PER_SEC, 3, None, None));
            if operation == Operation::Percentile {
                for i in 0..50_000 {
                    let _ = histogram.increment(i, 1);
                }
            }
            for mut tid in 0..threads {
                let histogram = histogram.clone();
                if contended {
                    tid = 1;
                }
                match operation {
                    Operation::Increment => {
                        thread_pool.push(thread::spawn(move || {
                            for _ in 0..(max / threads) {
                                let _ = histogram.increment(tid as u64 * 1_000_000, 1);
                            }
                        }));
                    }
                    Operation::Percentile => {
                        thread_pool.push(thread::spawn(move || {
                            for _ in 0..(max / threads) {
                                let _ = histogram.percentile(1.0);
                            }
                        }));
                    }
                }
            }
        }
    }
    for thread in thread_pool {
        thread.join().unwrap();
    }
    let t1 = time::Instant::now();
    (t1 - t0).as_secs() as f64 + ((t1 - t0).subsec_nanos() as f64 / NS_PER_SEC as f64)
}
