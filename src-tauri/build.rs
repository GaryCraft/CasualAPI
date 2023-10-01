use http_req::request;
use std::fs;
use std::io::{Cursor, Read};
use std::path::Path;

//const API_LATEST_VERSION_URL: &'static str = "https://api.github.com/repos/fatedier/frp/releases/latest";
const DOWNLOAD_BASE_URL: &'static str = "https://github.com/fatedier/frp/releases/download/";
const VERSION: &'static str = "0.51.3";

#[cfg(all(windows, target_arch = "x86_64"))]
const DOUBLE: &'static str = "windows_amd64";

#[cfg(all(windows, target_arch = "x86"))]
const DOUBLE: &'static str = "windows_386";

#[cfg(all(windows, target_arch = "arm64"))]
const DOUBLE: &'static str = "windows_arm64";

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
const DOUBLE: &'static str = "linux_amd64";

#[cfg(all(target_os = "linux", target_arch = "x86"))]
const DOUBLE: &'static str = "linux_386";

#[cfg(all(target_os = "linux", target_arch = "aarch64"))]
const DOUBLE: &'static str = "linux_arm64";

#[cfg(not(windows))]
const FORMAT: &'static str = "tar.gz";

#[cfg(windows)]
const FORMAT: &'static str = "zip";

fn main() {
    //download_frp();
    tauri_build::build()
}

fn download_frp() {
    //use std::io::{Read, Write};
    use flate2::read::GzDecoder;
    use tar::Archive;
    use zip::ZipArchive;
    // Get FRP from github release, DOWNLOAD_BASE_URL + VERSION + "/frp_" + VERSION + "_" + DOUBLE + ".zip"
    // and put it in [cargo dir]/binaries/frp
    // and put frpc.ini in [cargo dir]/binaries/frp

    let install_dir = get_install_dir();
    let frp_dir = install_dir.clone() + "/frp";

    // Create binaries/frp directory
    fs::create_dir_all(&frp_dir).unwrap();

    // Download frp
    let url = format!(
        "{}v{}/frp_{}_{}.{}",
        DOWNLOAD_BASE_URL, VERSION, VERSION, DOUBLE, FORMAT
    );
    match FORMAT {
        "zip" => {
            let frp_zip_buffer = download_zip(&url);
            let mut zip_archive =
                (ZipArchive::new(frp_zip_buffer)).expect("Failed to open zip file");
            // Extract frp
            for i in 0..zip_archive.len() {
                let mut file = zip_archive.by_index(i).unwrap();
                let entry_name = file.name().to_string();
                let mut file_buffer = Vec::new();
                file.read_to_end(&mut file_buffer).unwrap();
                let mut file_path = Path::new(&frp_dir).to_path_buf();
                file_path.push(&entry_name);
                println!(
                    "Extracting {} to {}",
                    entry_name,
                    file_path.to_str().unwrap()
                );
                if file_path.is_dir() {
                    fs::create_dir_all(&file_path).unwrap();
                } else {
                    fs::write(&file_path, file_buffer).unwrap();
                }
            }
        }
        "tar.gz" => {
            let frp_targz_buffer = download_tar(&url);
            let frp_tar_buffer = GzDecoder::new(frp_targz_buffer);
            let mut tar_archive = Archive::new(frp_tar_buffer);
            let entries = tar_archive.entries().expect("Failed to read tar file");
            for entry_result in entries {
                let mut entry = entry_result.unwrap();
                let entry_name = entry.path().unwrap().to_str().unwrap().to_string();
                // remove the trailing slash from the entry name
                let entry_name = entry_name.trim_end_matches('/');
                let mut file_buffer = Vec::new();
                entry.read_to_end(&mut file_buffer).unwrap();
                let mut file_path = Path::new(&frp_dir).to_path_buf();
                file_path.push(&entry_name);
				file_path.pop();
                println!(
                    "Extracting {} to {}",
                    entry_name,
                    file_path.to_str().unwrap()
                );
                if file_path.is_dir() {
                    fs::create_dir_all(&file_path).unwrap();
                } else {
                    fs::write(&file_path, file_buffer).unwrap();
                }
            }
        }
        _ => {
            println!("Unknown format: {}", FORMAT);
            std::process::exit(1);
        }
    }
}

fn download_zip(url: &str) -> Cursor<Vec<u8>> {
    match try_download(&url) {
        Ok(compressed_file) => compressed_file,
        Err(e) => {
            println!("Error downloading {}: {}", url, e);
            std::process::exit(1);
        }
    }
}

fn download_tar(url: &str) -> Cursor<Vec<u8>> {
    match try_download(&url) {
        Ok(compressed_file) => compressed_file,
        Err(e) => {
            println!("Error downloading {}: {}", url, e);
            std::process::exit(1);
        }
    }
}

fn get_install_dir() -> String {
    let mut install_dir = std::env::current_dir().unwrap();
    install_dir.push("binaries");
    install_dir.push("frp");
    install_dir.to_str().unwrap().to_string()
}

/// Try to download the specified URL into a buffer which is returned.
fn try_download(url: &str) -> Result<Cursor<Vec<u8>>, String> {
    // Send GET request
    let response = request::get(url).map_err(|error| error.to_string())?;

    // Only accept 2xx status codes
    if response.status_code() < 200 || response.status_code() >= 400 {
        return Err(format!("Download error: HTTP {}", response.status_code()));
    }
    if response.status_code() >= 300 {
        print!("Warning: HTTP {}", response.status_code());
        let location = response.headers().get("Location").unwrap();
        println!(" Redirecting to {}", location);
        return try_download(location);
    }
    let resp_body = response.body();
    let buffer = resp_body.to_vec();

    Ok(Cursor::new(buffer))
}
