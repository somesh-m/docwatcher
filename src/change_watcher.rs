use std::{
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
    time::Duration,
};

use notify::{EventKind, RecursiveMode};
use notify_debouncer_full::{DebounceEventResult, new_debouncer};
use tokio::sync::mpsc;

/// Watches `root` until the future is dropped or the process exits.
pub async fn run(root: PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    // Use absolute paths everywhere so they're consistent as keys in the hash DB.
    let root = root.canonicalize()?;

    let (tx, mut rx) = mpsc::unbounded_channel::<Vec<PathBuf>>();

    // This closure runs on notify's own thread, so keep it tiny:
    // classify the events and send paths over the channel.
    let mut debouncer = new_debouncer(
        Duration::from_secs(1),
        None,
        move |result: DebounceEventResult| match result {
            Ok(events) => {
                let mut changed = HashSet::new();
                for ev in events {
                    if matches!(
                        ev.kind,
                        EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_)
                    ) {
                        changed.extend(ev.paths.iter().cloned());
                    }
                }
                if !changed.is_empty() {
                    let _ = tx.send(changed.into_iter().collect());
                }
            }
            Err(errors) => errors.iter().for_each(|e| eprintln!("watch error: {e:?}")),
        },
    )?;

    debouncer.watch(&root, RecursiveMode::Recursive)?;
    // Some versions of the crate want debouncer.watcher().watch(...) instead.
    // The debouncer must stay alive for events to keep flowing, and it does
    // here because it lives until this function returns.

    while let Some(paths) = rx.recv().await {
        for path in paths {
            handle_path(path).await;
        }
    }
    Ok(())
}

async fn handle_path(path: PathBuf) {
    if !is_wanted(&path) {
        return;
    }
    if path.exists() {
        // new or changed: hash it, compare with stored hash, re-ingest if different
        println!("Update data: {path:?}");
    } else {
        // gone: delete all chunks for this path
        println!("File removed: {path:?}");
    }
}

fn is_wanted(path: &Path) -> bool {
    let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
    if name.starts_with('.')
        || name.starts_with("~$")
        || name.ends_with('~')
        || name.ends_with(".swp")
        || name.ends_with(".tmp")
    {
        return false;
    }
    matches!(
        path.extension()
            .and_then(|e| e.to_str())
            .map(str::to_ascii_lowercase)
            .as_deref(),
        Some("md" | "txt" | "pdf" | "docx")
    )
}

pub fn hash_file(path: impl AsRef<Path>) -> std::io::Result<String> {
    let mut hasher = blake3::Hasher::new();
    hasher.update_reader(fs::File::open(path)?)?;
    Ok(hasher.finalize().to_hex().to_string())
}
