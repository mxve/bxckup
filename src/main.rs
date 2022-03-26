// Hide console by default on windows
#![windows_subsystem = "windows"]
#![allow(dead_code)]

use serde_derive::Deserialize;
use std::{fs, path::Path};
use walkdir::WalkDir;
use winapi::um::wincon::{AttachConsole, FreeConsole, ATTACH_PARENT_PROCESS};

#[derive(Debug, Deserialize)]
struct Config {
    task: Vec<TaskConfig>,
}

#[derive(Debug, Deserialize)]
struct TaskConfig {
    source: String,
    target: String,
    exclude: Vec<String>,
}

fn load_config(path: &Path) -> Config {
    let cfg_file = fs::read_to_string(path).unwrap_or_else(|_| "".to_string());
    let cfg: Config = toml::from_str(&cfg_file).unwrap_or_else(|error| {
        println!("Error parsing config: {}", error);
        std::process::exit(1);
    });

    cfg
}

fn iterate_tasks(config: Config) {
    for task in config.task {
        handle_task(task);
    }
}

fn copy_file(source_path: &Path, target_path: &Path) {
    fs::copy(source_path, target_path).unwrap();
    println!(
        "Copied {} to {}",
        source_path.display(),
        target_path.display()
    );
}

fn handle_task(task: TaskConfig) {
    println!("{} -> {}", task.source, task.target);

    'files: for file in WalkDir::new(&task.source) {
        let file = file.unwrap();
        let source_path = file.path();

        for exclude in &task.exclude {
            if source_path.to_str().unwrap().contains(exclude) {
                println!("Skipping excluded file {}", source_path.to_str().unwrap());
                continue 'files;
            }
        }

        if !source_path.is_file() {
            continue;
        }

        let relative_path = source_path.strip_prefix(&task.source).unwrap();
        let target_path = Path::join(Path::new(&task.target), relative_path);
        let target_dir = target_path.parent().unwrap();

        if target_path.exists() {
            let source_checksum = crc32fast::hash(&fs::read(&source_path).unwrap());
            let target_checksum = crc32fast::hash(&fs::read(&target_path).unwrap());

            if source_checksum == target_checksum {
                continue;
            }

            println!("File changed: {}", relative_path.to_str().unwrap());
        } else {
            fs::create_dir_all(target_dir).unwrap();
        }

        copy_file(source_path, &target_path);
    }
}

fn main() {
    // Attach new console to print stdout
    unsafe {
        FreeConsole();
        AttachConsole(ATTACH_PARENT_PROCESS);
    }

    let config = load_config(Path::new("config.toml"));
    iterate_tasks(config);
}