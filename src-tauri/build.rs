use std::process::Command;

fn main() {
    tauri_build::build();

    #[cfg(target_os = "macos")]
    {
        // macOS 12+ includes Swift concurrency runtime in the system dyld cache,
        // so we only need /usr/lib/swift as rpath. On older macOS, we need the
        // CommandLineTools rpaths to find the Swift runtime.
        let macos_major = Command::new("sw_vers")
            .arg("-productVersion")
            .output()
            .ok()
            .and_then(|o| {
                let ver = String::from_utf8_lossy(&o.stdout).trim().to_string();
                ver.split('.').next()?.parse::<u32>().ok()
            });

        if macos_major.is_none_or(|v| v < 12) {
            if let Ok(output) = Command::new("xcode-select").arg("-p").output() {
                if output.status.success() {
                    let dev_path = String::from_utf8_lossy(&output.stdout).trim().to_string();
                    println!("cargo:rustc-link-arg=-Wl,-rpath,{dev_path}/usr/lib/swift-5.5/macosx");
                    println!("cargo:rustc-link-arg=-Wl,-rpath,{dev_path}/usr/lib/swift/macosx");
                }
            }
        }

        // Always add the system Swift lib path for linking and runtime resolution.
        println!("cargo:rustc-link-search=native=/usr/lib/swift");
        println!("cargo:rustc-link-arg=-Wl,-rpath,/usr/lib/swift");
    }
}
