use core::mem;
use core::sync::atomic::AtomicU8;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::Ordering;
use std::sync::{Arc, Once};
use std::thread;

use dashmap::DashMap;
use env_logger::Env;
use futures::channel::oneshot;
use futures::channel::oneshot::Sender;
use futures::SinkExt;
use tokio::runtime::Runtime;
use tokio::sync::broadcast::RecvError;
use warp::ws::Message;
use warp::{path, Filter, Reply};

use crate::mdnotes::MdNotes;
use crate::{warp_fs, MdNotesError};

static STARTUP: Once = Once::new();

pub struct MdNotesRuntime {
    note_inc: AtomicU8,
    notes: Arc<DashMap<u8, MdNotes>>,
    server_address: SocketAddr,
    shutdown: Option<Sender<()>>,
}

const PATH_ENV: &str = "PATH";

impl MdNotesRuntime {
    pub fn new() -> Result<MdNotesRuntime, MdNotesError> {
        STARTUP.call_once(|| {
            env_logger::init_from_env(
                Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "info"),
            );

            setup_environment();
        });

        let notes: Arc<DashMap<u8, MdNotes>> = Arc::new(DashMap::new());
        let route_notes = notes.clone();

        let static_route = warp::path::param()
            .and(warp::path("static"))
            .and(warp::path::tail())
            .and_then(move |raw_notes_id: String, tail: path::Tail| {
                let route_notes = route_notes.clone();

                async move {
                    let notes_id = raw_notes_id.parse::<u8>().unwrap();

                    match route_notes.get(&notes_id) {
                        Some(note) => warp_fs::serve_file(&note.html_dir, tail).await,
                        None => Err(warp::reject()),
                    }
                }
            });

        let route_notes = notes.clone();

        let ws_route = warp::path::param()
            .and(warp::path("ws"))
            .and(warp::ws())
            .map(move |raw_notes_id: String, ws: warp::ws::Ws| {
                let notes_id = raw_notes_id.parse::<u8>().unwrap();
                let route_notes = route_notes.clone();

                ws.on_upgrade(move |mut websocket| async move {
                    match route_notes.get(&notes_id) {
                        Some(note) => {
                            let mut receiver = note.get_ws_receiver();
                            // drop our lock on the note
                            mem::drop(note);

                            loop {
                                // wait for the not receiver to tell us to reload
                                match receiver.recv().await {
                                    Ok(event) => match websocket.send(Message::text(event)).await {
                                        Ok(_) => (),
                                        Err(e) => {
                                            warn!("ws send error: {}", e);
                                            break;
                                        }
                                    },
                                    Err(RecvError::Lagged(_)) => (), // we don't care if we're lagging
                                    Err(RecvError::Closed) => break, // we're done broadcasting so break out
                                }
                            }
                        }
                        None => match websocket.close().await {
                            Ok(_) => (),
                            Err(e) => log::warn!("Error closing web socket: {}", e),
                        },
                    };

                    println!("done with route get");
                })
            });

        let routes = static_route.or(ws_route);

        let (address, shutdown) = spawn_background_server(routes)?;

        Ok(MdNotesRuntime {
            note_inc: AtomicU8::new(1),
            notes,
            server_address: address,
            shutdown: Some(shutdown),
        })
    }

    pub fn server_port(&self) -> u16 {
        self.server_address.port()
    }

    pub fn open_notes(&self, book_dir: PathBuf) -> u8 {
        let notes_id = self.note_inc.fetch_add(1, Ordering::Relaxed);

        info!(
            "Loading notes: {} @ {}",
            notes_id,
            book_dir.to_string_lossy()
        );

        let notes = MdNotes::new(notes_id, book_dir, self.server_port()).unwrap();

        self.notes.insert(notes_id, notes);

        notes_id
    }

    pub fn close_notes(&self, note_id: u8) {
        if let Some((_, notes)) = self.notes.remove(&note_id) {
            mem::drop(notes);

            info!("Closed notes: {}", note_id);
        } else {
            warn!("Tried to close invalid note_id: {}", note_id);
        }
    }
}

impl Drop for MdNotesRuntime {
    fn drop(&mut self) {
        // signal our server to shutdown
        mem::replace(&mut self.shutdown, None).map(|shutdown| shutdown.send(()));
    }
}

fn spawn_background_server<F>(routes: F) -> Result<(SocketAddr, Sender<()>), MdNotesError>
where
    F: Filter + Clone + Send + Sync + 'static,
    F::Extract: Reply,
{
    let (background_sender, foreground_receiver) = oneshot::channel();
    let (shutdown, shutdown_signal) = oneshot::channel::<()>();

    thread::spawn(|| {
        let runtime_result = Runtime::new();

        match runtime_result {
            Ok(mut runtime) => {
                let (address, startup) = runtime.enter(|| {
                    warp::serve(routes).bind_with_graceful_shutdown(([127, 0, 0, 1], 0), async {
                        shutdown_signal.await.ok();
                    })
                });

                info!("Server started up");

                background_sender
                    .send(Ok(address))
                    .expect("Our channel to the foreground should always be open");

                runtime.block_on(startup);

                info!("Server stopped, shutting down runtime");
            }
            Err(e) => {
                background_sender
                    .send(Err(format!("{}", e)))
                    .expect("Our channel to the foreground should always be open");
            }
        };
    });

    let address =
        futures::executor::block_on(foreground_receiver).map_err(|e| format!("{}", e))??;

    Ok((address, shutdown))
}

/// Mac applications get their environment from launchctl, but that kind of sucks. So attempt to
/// load in our known possible environments and use their parameters instead.
///
/// This prioritizes loading our environment in this order:
/// * zsh
/// * bash
/// * fall back on setting the path for Cargo and Homebrew
fn setup_environment() {
    fn parse_environment(env_str: String, environment: &mut HashMap<String, String>) -> bool {
        let mut found = false;
        for raw_line in env_str.split('\n') {
            let line: Vec<_> = raw_line.split('=').collect();

            if line.len() == 2 {
                found = true;
                environment.insert(line[0].into(), line[1].into());
            }
        }

        found
    }

    // if we can't find a home directory, we don't really know what to set as our path
    if let Some(home_dir) = dirs::home_dir() {
        let mut environment = HashMap::new();

        // attempt to load our environment
        for command in &["zsh", "bash"] {
            match Command::new(command)
                .arg("-c") // run the command
                .arg("-l") // run as a login shell
                .arg("printenv")
                .output()
                .map_err(|e| format!("{}", e))
                .and_then(|out| {
                    std::str::from_utf8(&out.stdout)
                        .map(String::from)
                        .map_err(|e| format!("{}", e))
                }) {
                Ok(env) => {
                    if parse_environment(env, &mut environment) {
                        // if we could parse our environment break out
                        break;
                    }
                }
                Err(e) => info!("Couldn't load our {} environment: {}", command, e),
            }
        }

        // if we still don't have any environment variables or a path, just update our path
        if environment.is_empty() || !environment.contains_key(PATH_ENV) {
            let cargo_path = format!("{}/.cargo/bin", home_dir.to_string_lossy());
            let path = if let Some(mut found_path) = std::env::var_os(PATH_ENV) {
                found_path.push(format!(":{}", cargo_path));
                found_path.to_string_lossy().into()
            } else {
                cargo_path
            };

            environment.insert(PATH_ENV.into(), path);
        }

        for (key, value) in environment {
            debug!("Setting environment: {}={}", key, value);
            std::env::set_var(key, value);
        }
    }
}
