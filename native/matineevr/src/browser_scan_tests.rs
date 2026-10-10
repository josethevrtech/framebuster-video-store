use super::*;
use crate::browser::tests::fixture;
use std::{
    io::Write,
    sync::{Condvar, atomic::AtomicUsize},
    time::{Duration, Instant, UNIX_EPOCH},
};

#[test]
fn eight_workers_preserve_sort_order() {
    let root = fixture();
    for i in (0..24).rev() {
        fs::write(root.join(format!("{i:02}")), []).unwrap();
    }
    let scanner = Scanner::default();
    let active = AtomicUsize::new(0);
    let peak = Mutex::new(0);
    let ready = Condvar::new();
    let entries = scanner
        .scan(&root, &AtomicBool::new(false), |_| {
            let count = active.fetch_add(1, Ordering::SeqCst) + 1;
            let mut highest = peak.lock().unwrap();
            *highest = (*highest).max(count);
            ready.notify_all();
            let (highest, timeout) = ready
                .wait_timeout_while(highest, Duration::from_secs(5), |n| *n < WORKERS)
                .unwrap();
            assert!(!timeout.timed_out(), "Eight probes did not overlap");
            drop(highest);
            active.fetch_sub(1, Ordering::SeqCst);
            Ok(true)
        })
        .unwrap();
    assert_eq!(*peak.lock().unwrap(), WORKERS);
    let expected: Vec<_> = (0..24).map(|i| root.join(format!("{i:02}"))).collect();
    assert_eq!(
        entries.into_iter().map(|e| e.path).collect::<Vec<_>>(),
        expected
    );
}

#[test]
fn caches_both_results_and_reprobes_changes_and_new_files() {
    let root = fixture();
    let video = root.join("no-extension");
    let other = root.join("fake.mp4");
    fs::write(&video, b"supported").unwrap();
    fs::write(&other, b"unsupported").unwrap();
    fs::create_dir(root.join("folder")).unwrap();
    let scanner = Scanner::default();
    let cancelled = AtomicBool::new(false);
    let calls = AtomicUsize::new(0);
    let probe = |path: &Path| {
        calls.fetch_add(1, Ordering::Relaxed);
        Ok(fs::read(path)? == b"supported")
    };
    assert_eq!(scanner.scan(&root, &cancelled, probe).unwrap().len(), 2);
    assert_eq!(scanner.scan(&root, &cancelled, probe).unwrap().len(), 2);
    assert_eq!(calls.load(Ordering::Relaxed), 2);
    fs::OpenOptions::new()
        .append(true)
        .open(&video)
        .unwrap()
        .write_all(b" changed")
        .unwrap();
    assert_eq!(scanner.scan(&root, &cancelled, probe).unwrap().len(), 1);
    assert_eq!(calls.load(Ordering::Relaxed), 3);
    fs::File::open(&other)
        .unwrap()
        .set_modified(UNIX_EPOCH)
        .unwrap();
    fs::write(root.join("new-video"), b"supported").unwrap();
    assert_eq!(scanner.scan(&root, &cancelled, probe).unwrap().len(), 2);
    assert_eq!(calls.load(Ordering::Relaxed), 5);
}

#[test]
fn cancellation_stops_new_probes_and_errors_are_not_cached() {
    let root = fixture();
    for i in 0..32 {
        fs::write(root.join(i.to_string()), []).unwrap();
    }
    let scanner = Scanner::default();
    let cancelled = AtomicBool::new(true);
    let calls = AtomicUsize::new(0);
    let probe = |_: &Path| {
        calls.fetch_add(1, Ordering::Relaxed);
        cancelled.store(true, Ordering::Relaxed);
        Ok(true)
    };
    assert!(scanner.scan(&root, &cancelled, probe).unwrap().is_empty());
    assert_eq!(calls.load(Ordering::Relaxed), 0);
    cancelled.store(false, Ordering::Relaxed);
    scanner.scan(&root, &cancelled, probe).unwrap();
    assert!((1..=WORKERS).contains(&calls.load(Ordering::Relaxed)));
    cancelled.store(false, Ordering::Relaxed);
    let entries = scanner
        .scan(&root, &cancelled, |_| anyhow::bail!("probe failed"))
        .unwrap();
    assert_eq!(entries.len(), calls.load(Ordering::Relaxed));
    assert_eq!(
        scanner
            .scan(&root, &cancelled, |_| {
                calls.fetch_add(1, Ordering::Relaxed);
                Ok(true)
            })
            .unwrap()
            .len(),
        32
    );
    assert_eq!(calls.load(Ordering::Relaxed), 32);
}

#[test]
#[ignore = "300-file scan timing with a simulated 10 ms metadata probe"]
fn scan_timing() {
    let root = fixture();
    for i in 0..300 {
        fs::write(root.join(i.to_string()), []).unwrap();
    }
    let scanner = Scanner::default();
    for round in 0..2 {
        let start = Instant::now();
        let entries = scanner
            .scan(&root, &AtomicBool::new(false), |_| {
                thread::sleep(Duration::from_millis(10));
                Ok(true)
            })
            .unwrap();
        assert_eq!(entries.len(), 300);
        eprintln!("Scan round {round}: {:?}", start.elapsed());
    }
}
