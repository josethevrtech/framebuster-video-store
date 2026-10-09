use super::*;
use crate::{
    hud_text::{Canvas, HEIGHT, WIDTH},
    options::Options,
};
use std::{
    fs,
    sync::atomic::AtomicUsize,
    time::{SystemTime, UNIX_EPOCH},
};

pub(crate) fn fixture() -> PathBuf {
    static NEXT_ID: AtomicUsize = AtomicUsize::new(0);
    let path = std::env::temp_dir().join(format!(
        "frame-browser-{}-{}-{}",
        std::process::id(),
        NEXT_ID.fetch_add(1, Ordering::Relaxed),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir(&path).unwrap();
    path
}

#[test]
fn listing_uses_content_and_sorts_directories_before_files() {
    let root = fixture();
    fs::create_dir(root.join("z-folder")).unwrap();
    fs::write(root.join("fake.mp4"), "unsupported").unwrap();
    fs::write(root.join("extensionless"), "supported").unwrap();
    fs::write(root.join("a.mkv"), "supported").unwrap();
    let scanner = Scanner::default();
    let cancelled = AtomicBool::new(false);
    let entries = scanner
        .scan(
            &root,
            &cancelled,
            |path| Ok(fs::read(path)? == b"supported"),
        )
        .unwrap();
    let names: Vec<_> = entries
        .iter()
        .map(|e| e.path.file_name().unwrap())
        .collect();
    assert_eq!(names, ["z-folder", "a.mkv", "extensionless"]);
    std::os::unix::fs::symlink(root.join("z-folder"), root.join("folder-link")).unwrap();
    std::os::unix::fs::symlink(root.join("a.mkv"), root.join("video-link")).unwrap();
    std::os::unix::fs::symlink(root.join("missing"), root.join("broken-link")).unwrap();
    assert_eq!(
        scanner
            .scan(
                &root,
                &cancelled,
                |path| Ok(fs::read(path)? == b"supported")
            )
            .unwrap()
            .len(),
        5
    );
    assert!(
        scanner
            .scan(&root.join("missing"), &cancelled, |_| Ok(true))
            .is_err()
    );
}

#[test]
fn navigation_loading_empty_directory_and_scan_errors() {
    let root = fixture();
    let (sender, receiver) = mpsc::channel();
    let mut browser = Browser {
        directory: root.clone(),
        entries: Vec::new(),
        selected: 0,
        message: String::new(),
        selections: HashMap::new(),
        pending: Some(receiver),
        scanner: Arc::default(),
        cancelled: Arc::default(),
    };
    browser.move_selection(-1);
    assert!(browser.open().is_none());
    assert!(!browser.poll());
    sender.send(Ok(Vec::new())).unwrap();
    assert!(browser.poll());
    assert!(!browser.loading());
    browser.move_selection(1);
    assert_eq!(browser.selected, 0);
    assert!(browser.open().is_none());
    browser.entries = ["a", "b", "c", "d", "e"]
        .iter()
        .map(|name| Entry {
            path: root.join(name),
            directory: false,
        })
        .collect();
    browser.move_selection(100);
    assert_eq!(browser.open(), Some(root.join("e")));
    browser.move_selection(-100);
    assert_eq!(browser.open(), Some(root.join("a")));
    browser.entries[0].directory = true;
    browser.entries[0].path = root.clone();
    let cancelled = browser.cancelled.clone();
    assert!(browser.open().is_none());
    assert!(cancelled.load(Ordering::Relaxed));
    assert!(browser.loading());
    assert_eq!(browser.entries[0].path, root.parent().unwrap());
    let (sender, receiver) = mpsc::channel();
    browser.pending = Some(receiver);
    sender
        .send(Err(anyhow::anyhow!("Permission denied")))
        .unwrap();
    assert!(browser.poll());
    assert!(browser.message.contains("Permission denied"));
    let mut canvas = Canvas::new(&root, [WIDTH, HEIGHT]);
    canvas.browser(&browser, false, 0.0);
    assert!(
        canvas
            .pixels
            .chunks_exact(4)
            .any(|pixel| pixel == [255, 160, 130, 255])
    );
}

#[test]
fn revisits_restore_paths_after_reordering_and_interrupted_scans() {
    let root = fixture();
    let mut browser = Browser::new(root.clone());
    let complete = |browser: &mut Browser, names: &[&str]| {
        let (sender, receiver) = mpsc::channel();
        browser.pending = Some(receiver);
        sender
            .send(Ok(names
                .iter()
                .map(|name| Entry {
                    path: browser.directory.join(name),
                    directory: true,
                })
                .collect()))
            .unwrap();
        assert!(browser.poll());
    };
    complete(&mut browser, &["a", "b", "c"]);
    browser.move_selection(2);
    assert!(browser.open().is_none());
    assert_eq!(browser.directory, root.join("c"));
    complete(&mut browser, &["first", "second"]);
    browser.move_selection(1);
    browser.parent();
    browser.enter(root.join("c"));
    complete(&mut browser, &["first", "second"]);
    assert_eq!(browser.selected, 2);
    browser.parent();
    complete(&mut browser, &["0", "a", "b", "c"]);
    assert_eq!(browser.selected, 4);
    browser.open();
    complete(&mut browser, &["first"]);
    assert_eq!(browser.selected, 1);
    browser.move_selection(-1);
    browser.open();
    complete(&mut browser, &["0", "a", "b", "c"]);
    assert_eq!(browser.selected, 4);
    browser.open();
    complete(&mut browser, &["first"]);
    assert_eq!(browser.selected, 0);
}

#[test]
fn cli_accepts_browser_startup_and_requires_files_for_probes() {
    let root = fixture();
    let parse = |args: &[&str]| Options::from_args(args.iter().map(|s| s.to_string()));
    assert!(parse(&[]).unwrap().unwrap().file.is_dir());
    let options = parse(&[root.to_str().unwrap()]).unwrap().unwrap();
    assert_eq!(options.file, root.canonicalize().unwrap());
    assert!(!options.hud);
    assert!(
        parse(&[root.to_str().unwrap(), "--hud"])
            .unwrap()
            .unwrap()
            .hud
    );
    assert!(parse(&[root.to_str().unwrap(), "--probe"]).is_err());
    assert!(parse(&[root.join("missing").to_str().unwrap()]).is_err());
}
