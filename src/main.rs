use ini::Ini;
use std::env;
use std::fs;
use std::path::{Path,PathBuf};
use std::sync::mpsc::channel;
use std::thread;

fn main() {
    // Read configuration
    let config = Ini::load_from_file("config.ini").unwrap();
    let log_section = config.section(Some("log")).unwrap();
    let filename = log_section.get("filename").unwrap();
    let filepath = log_section.get("filepath").unwrap();

    let file_path = Path::new(filepath).join(filename);
    let temp_directory = env::temp_dir();
    let index_dir_opt = temp_directory.to_str();
    let index_dir: &str;
    index_dir = match index_dir_opt {
        Some(p) => p,
        None => "/tmp",
    };
    let index_path: String = format!("{}/{}.idx", index_dir, filename);
    let index_file_path = Path::new(&index_path);

    // Read the index of the last line read from the index file
    let mut last_line_read = 0;
    if index_file_path.exists() {
        let index_str = fs::read_to_string(index_file_path).unwrap();
        last_line_read = index_str.trim().parse().unwrap();
    }

    let file_size_indicator = check_file_size(&file_path);
    let (tx, rx) = channel();

    // Spawn a new thread to listen for new writes to the file
    thread::spawn(move || {
        let index_path_clone = index_path.clone();
        let file_path_clone = file_path.clone();
        loop {
            if check_file_size(&file_path_clone) < file_size_indicator {
                write_index(&index_path_clone, 0)
            }

            let lines: Vec<String> = fs::read_to_string(&file_path_clone)
                .unwrap()
                .lines()
                .skip(last_line_read)
                .map(|x| x.to_string())
                .collect();
            if !lines.is_empty() {
                last_line_read += lines.len();
                // Update the index file with the new last line read
                write_index(&index_path_clone, last_line_read);

                tx.send(lines).unwrap();
            }
        }
    });

    // Main thread listens for new lines and processes them
    loop {
        let lines = rx.recv().unwrap();
        for line in lines {
            println!("{}", line);
        }
    }
}

/// Check if file size is shrinking to detect de log file rotation
/// 
fn check_file_size(file: &PathBuf) -> u64 {
    // Check the file size
    let metadata = fs::metadata(file).unwrap();
    return metadata.len()
}

/// Write `index` to `index_file` String representing a path to an index file
/// 
fn write_index(index_file: &String, index: usize) {
    // Update the index file with the new last line read
    let index_file_path = Path::new(&index_file);
    fs::write(index_file_path, index.to_string()).unwrap();
}
