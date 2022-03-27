// Hide console by default on windows
#![windows_subsystem = "windows"]
#![allow(dead_code)]

use serde_derive::Deserialize;
use std::{fs, path::Path};
use winapi::um::wincon::{AttachConsole, FreeConsole, ATTACH_PARENT_PROCESS};

mod args;

#[derive(Debug, Deserialize)]
struct Config {
    task: Vec<TaskConfig>,
}

#[derive(Debug, Deserialize)]
struct TaskConfig {
    source: String,
    target: String,
    exclude: Vec<String>,
    remove_deleted: bool,
}

fn load_config(path: &Path) -> Config {
    let cfg_file = fs::read_to_string(path).unwrap_or_else(|_| "".to_string());
    let cfg: Config = toml::from_str(&cfg_file).unwrap_or_else(|error| {
        println!("Error parsing config {}", error);
        std::process::exit(1);
    });

    cfg
}

fn copy_file(source_path: &Path, target_path: &Path) {
    fs::copy(source_path, target_path).unwrap();
    println!(
        "Copied {} to {}",
        source_path.display(),
        target_path.display()
    );
}

fn backup_file(source_path: &Path, target_path: &Path, exclude: &Vec<String>) {
    let target_dir = target_path.parent().unwrap();

    for exclude in exclude {
        if source_path.to_str().unwrap().contains(exclude) {
            println!("Skipping excluded file {}", source_path.to_str().unwrap());
            return;
        }
    }

    if !source_path.is_file() {
        return;
    }

    if target_path.exists() {
        let source_checksum = crc32fast::hash(&fs::read(&source_path).unwrap());
        let target_checksum = crc32fast::hash(&fs::read(&target_path).unwrap());

        if source_checksum == target_checksum {
            return;
        }

        println!("File changed {}", &source_path.to_str().unwrap());
    } else {
        fs::create_dir_all(target_dir).unwrap();
    }

    copy_file(source_path, &target_path);
}

fn iterate_tasks(config: Config) {
    for task in config.task {
        handle_task(task);
    }
}

fn handle_task(task: TaskConfig) {
    println!("{} -> {}", task.source, task.target);

    for file in walkdir::WalkDir::new(&task.source) {
        let file = file.unwrap();
        let source_path = file.path();
        let relative_path = source_path.strip_prefix(&task.source).unwrap();
        let target_path = Path::join(Path::new(&task.target), relative_path);
        backup_file(source_path, &target_path, &task.exclude);
    }

    if task.remove_deleted {
        for file in walkdir::WalkDir::new(&task.target) {
            
            let file = file.unwrap();
            let target_path = file.path();
            let relative_path = target_path.strip_prefix(&task.target).unwrap();
            let source_path = Path::join(Path::new(&task.source), relative_path);


            if !source_path.exists() {
                println!("Deleting {}", target_path.to_str().unwrap());

                if !target_path.is_file() {
                    fs::remove_dir_all(target_path).unwrap_or_else(|error| {
                        println!("Error deleting {}", error);
                    });
                    continue
                }

                fs::remove_file(target_path).unwrap_or_else(|error| {
                    println!("Error deleting {}", error);
                });
            }
        }
    }
}

fn main() {
    // Attach new console to print stdout
    unsafe {
        FreeConsole();
        AttachConsole(ATTACH_PARENT_PROCESS);
    }

    let args = args::get();
    println!("{}", args.config.to_str().unwrap());

    let config = load_config(Path::new(&args.config));
    iterate_tasks(config);
}
