use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::RecvTimeoutError;
use std::sync::{mpsc, Arc};
use std::time::Duration;
use std::{fs, thread};

use mdbook::errors::Error as MDBookError;
use mdbook::MDBook;
use notify::{DebouncedEvent, RecommendedWatcher, RecursiveMode, Watcher};
use tokio::sync::broadcast;
use tokio::sync::broadcast::{Receiver, Sender};

use crate::MdNotesError;

pub struct MdNotes {
    pub html_dir: PathBuf,
    shutdown_hook: Arc<AtomicBool>,
    broadcast: Sender<String>,
}

impl MdNotes {
    pub fn new(id: u8, book_dir: PathBuf, port: u16) -> Result<MdNotes, String> {
        let livereload_url = format!("ws://localhost:{}/{}/ws", port, id);
        let book = build_book(&book_dir, &livereload_url)
            .map_err(|e| format!("Couldn't rebuild the build: {}", e))?;
        let html_dir = book.build_dir_for("html");

        // we don't care about this initial receiver
        let (sender, _) = broadcast::channel::<String>(10);

        let shutdown_hook = start_fs_watcher(&book, livereload_url, sender.clone())?;

        Ok(MdNotes {
            html_dir,
            shutdown_hook,
            broadcast: sender,
        })
    }

    pub fn get_ws_receiver(&self) -> Receiver<String> {
        self.broadcast.subscribe()
    }
}

impl Drop for MdNotes {
    fn drop(&mut self) {
        // signal our fs watcher to shutdown
        self.shutdown_hook.store(true, Ordering::Relaxed);
    }
}

fn start_fs_watcher(
    book: &MDBook,
    livereload_url: String,
    broadcast: Sender<String>,
) -> Result<Arc<AtomicBool>, MdNotesError> {
    let book_dir = book.root.clone();
    let source_dir = book.source_dir();
    let theme_dir = book.theme_dir();

    let (sender, receiver) = mpsc::channel();

    let mut watcher = RecommendedWatcher::new(sender, Duration::from_millis(100))
        .map_err(|e| format!("Error watching file system: {}", e))?;

    let mut watching_something = false;
    for watch_path in &[&source_dir, &theme_dir, &book.root.join("book.toml")] {
        match watcher.watch(watch_path, RecursiveMode::Recursive) {
            Ok(_) => watching_something = true,
            Err(e) => warn!("Couldn't watch book directory '{:?}': {}", watch_path, e),
        }
    }

    if watching_something {
        let shutdown = Arc::new(AtomicBool::new(false));
        let fs_shutdown = shutdown.clone();
        thread::spawn(move || {
            // take ownership of watcher in this thread so that we can drop it when we're done
            let _ = watcher;
            let reload_event = "reload".to_string();

            // check if we should shutdown every loop
            while !fs_shutdown.load(Ordering::Relaxed) {
                // only wait for 1 second, we want to make sure to check our shutdown status
                match receiver.recv_timeout(Duration::from_secs(1)) {
                    Ok(first_event) => {
                        thread::sleep(Duration::from_millis(50));
                        let other_events = receiver.try_iter();

                        let all_events = std::iter::once(first_event).chain(other_events);

                        let paths = all_events.filter_map(|event| {
                            trace!("Received filesystem event: {:?}", event);

                            match event {
                                DebouncedEvent::Create(path)
                                | DebouncedEvent::Write(path)
                                | DebouncedEvent::Remove(path)
                                | DebouncedEvent::Rename(_, path) => Some(path),
                                _ => None,
                            }
                        });

                        if found_unignored_files(paths, &book_dir) {
                            debug!("Reloading book: {:?}", book_dir);

                            match build_book(&book_dir, &livereload_url) {
                                Ok(_) => (),
                                Err(e) => warn!("Couldn't rebuild the book: {}", e),
                            }

                            // according to the doc, an error means there were no receivers, so ignore it
                            let _ = broadcast.send(reload_event.clone());
                        }
                    }
                    Err(RecvTimeoutError::Timeout) => (), // ignore timeouts
                    Err(RecvTimeoutError::Disconnected) => {
                        // on disconnect, we can stop checking for fs events
                        break;
                    }
                }
            }
            info!("Stopped watching the fs for {:?}", source_dir);
        });

        Ok(shutdown)
    } else {
        // Nothing to watch so just start in shutdown
        warn!("Couldn't watch the filesystem for {:?}", book_dir);
        Ok(Arc::new(AtomicBool::new(true)))
    }
}

fn found_unignored_files<I>(mut paths: I, book_dir: &PathBuf) -> bool
where
    I: Iterator<Item = PathBuf>,
{
    if let Some(gitignore_path) = book_dir
        .ancestors()
        .map(|p| p.join(".gitignore"))
        .find(|p| p.exists())
        .map(|p| fs::canonicalize(p).expect("This should exist and therefore not error"))
    {
        match gitignore::File::new(&gitignore_path) {
            Ok(exclusion_checker) => paths.any(|path| match exclusion_checker.is_excluded(&path) {
                Ok(excluded) => !excluded,
                Err(e) => {
                    warn!(
                        "Found an error when checking .gitignore exclusion for {:?}: {}",
                        path, e
                    );
                    true
                }
            }),
            Err(e) => {
                warn!("Couldn't parse our .gitignore: {}", e);

                paths.count() > 0
            }
        }
    } else {
        paths.count() > 0
    }
}

fn build_book(book_dir: &PathBuf, livereload_url: &str) -> Result<MDBook, MDBookError> {
    let mut book = MDBook::load(&book_dir)?;

    book.config
        .set("output.html.livereload-url", livereload_url)?;

    book.build()?;

    Ok(book)
}
