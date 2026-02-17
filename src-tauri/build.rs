use std::process::Command;

fn main() {
    tauri_build::build();

    #[cfg(target_os = "macos")]
    {
        let system_lib = std::path::Path::new("/usr/lib/swift/libswift_Concurrency.dylib");
        if !system_lib.exists() {
            if let Ok(output) = Command::new("xcode-select").arg("-p").output() {
                if output.status.success() {
                    let dev_path = String::from_utf8_lossy(&output.stdout).trim().to_string();
                    println!("cargo:rustc-link-arg=-Wl,-rpath,{dev_path}/usr/lib/swift-5.5/macosx");
                    println!("cargo:rustc-link-arg=-Wl,-rpath,{dev_path}/usr/lib/swift/macosx");
                }
            }
        }
        println!("cargo:rustc-link-search=native=/usr/lib/swift");
    }
}
