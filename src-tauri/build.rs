use std::process::Command;

fn main() {
    tauri_build::build();

    // Add rpath for Swift runtime libraries (needed by screencapturekit crate).
    // CommandLineTools installs Swift libs under usr/lib/ directly, while
    // Xcode.app uses Toolchains/XcodeDefault.xctoolchain/usr/lib/.
    if let Ok(output) = Command::new("xcode-select").arg("-p").output() {
        if output.status.success() {
            let dev_path = String::from_utf8_lossy(&output.stdout).trim().to_string();
            println!("cargo:rustc-link-arg=-Wl,-rpath,{dev_path}/usr/lib/swift-5.5/macosx");
            println!("cargo:rustc-link-arg=-Wl,-rpath,{dev_path}/usr/lib/swift/macosx");
        }
    }

    // Also add the system Swift lib path for runtime linking
    println!("cargo:rustc-link-search=native=/usr/lib/swift");
}
