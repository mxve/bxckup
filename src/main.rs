// Hide console by default on windows
#![windows_subsystem = "windows"]

use serde_derive::Deserialize;
use std::{fs, path::Path};
use winapi::um::wincon::{AttachConsole, FreeConsole, ATTACH_PARENT_PROCESS};

mod args;

// Config file structs
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
    most_recent_only: bool,
    skip_crc: bool,
}

// Parse config file to structs
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

fn crc32_files(file1: &Path, file2: &Path) -> bool {
    let checksum1 = crc32fast::hash(&fs::read(file1).unwrap());
    let checksum2 = crc32fast::hash(&fs::read(file2).unwrap());
    if checksum1 == checksum2 {
        return true;
    }
    false
}

fn backup_file(source_path: &Path, target_path: &Path, exclude: &[String], skip_crc: &bool) {
    let target_dir = target_path.parent().unwrap();
    for e in exclude {
        if source_path.to_str().unwrap().contains(e) {
            //println!("Skipping excluded file {}", source_path.to_str().unwrap());
            return;
        }
    }
    if !source_path.is_file() {
        return;
    }

    if target_path.exists() {
        if *skip_crc {
            return;
        }
        if crc32_files(source_path, target_path) {
            println!(
                "File matches on source and target {}",
                source_path.to_str().unwrap()
            );
            return;
        }
    } else {
        fs::create_dir_all(target_dir).unwrap_or_else(|error| {
            println!("Error creating target directory {}", error);
            std::process::exit(1);
        });
    }

    println!(
        "Copying {} to {}",
        source_path.display(),
        target_path.display()
    );

    copy_file(source_path, target_path);
}

// iterate config.task vec
fn iterate_tasks(config: Config) {
    for task in config.task {
        handle_task(task);
    }
}

// backup according to rules in task
fn handle_task(task: TaskConfig) {
    println!("{} -> {}", task.source, task.target);

    if task.most_recent_only {
        let last_modified_file = std::fs::read_dir(&task.source)
            .expect("Couldn't access local directory")
            .flatten()
            .filter(|f| f.metadata().unwrap().is_file())
            .max_by_key(|x| x.metadata().unwrap().modified().unwrap());
        match last_modified_file {
            Some(file) => {
                let source_path = file.path();
                let relative_path = source_path.strip_prefix(&task.source).unwrap();
                let target_path = Path::join(Path::new(&task.target), relative_path);
                backup_file(&source_path, &target_path, &task.exclude, &task.skip_crc);
            }
            None => {
                println!("No files found in {}", task.source);
            }
        }
    } else {
        for file in walkdir::WalkDir::new(&task.source) {
            let file = file.unwrap();
            let source_path = file.path();
            let relative_path = source_path.strip_prefix(&task.source).unwrap();
            let target_path = Path::join(Path::new(&task.target), relative_path);

            backup_file(source_path, &target_path, &task.exclude, &task.skip_crc);
        }
    }

    // delete files that have been deleted from source dir from target dir
    if task.remove_deleted {
        for file in walkdir::WalkDir::new(&task.target) {
            let file = file.unwrap();
            let target_path = file.path();
            let relative_path = target_path.strip_prefix(&task.target).unwrap();
            let source_path = Path::join(Path::new(&task.source), relative_path);

            if !source_path.exists() {
                println!("Deleting {}", target_path.to_str().unwrap());
                // assume that everything that isn't a file is a directory
                // doing directories first reduces iterations
                if !target_path.is_file() {
                    fs::remove_dir_all(target_path).unwrap_or_else(|error| {
                        println!("Error deleting {}", error);
                    });
                    continue;
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
    let config = load_config(Path::new(&args.config));
    iterate_tasks(config);
}
