use std::path::PathBuf;

use docwatcher::change_watcher;
use rig::{AgentBuilder, providers::ollama};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = ollama::Ollama::new();

    let agent = AgentBuilder::new(client.completion("qwen3:4b"))
        .preamble("You are document tracking agent")
        .build();
    let watch_folder_path = String::from("/home/sohmesh-mohan/docwatcher/watch");
    change_watcher::run(PathBuf::from(watch_folder_path)).await?;
    println!("After setting the loop");
    Ok(())
}
