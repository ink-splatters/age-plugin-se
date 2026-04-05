//! Build script for apple-secure-enclave.
//!
//! Links the required `macOS` frameworks.

fn main() {
    #[cfg(target_os = "macos")]
    {
        // Link the LocalAuthentication framework for LAContext
        println!("cargo::rustc-link-lib=framework=LocalAuthentication");

        // Link the Foundation framework
        println!("cargo::rustc-link-lib=framework=Foundation");

        // Security framework is already linked by security-framework-sys,
        // but we explicitly link it here for clarity
        println!("cargo::rustc-link-lib=framework=Security");

        // Get SDK path for library search
        if let Ok(output) = std::process::Command::new("xcrun")
            .args(["--show-sdk-path"])
            .output()
        {
            if output.status.success() {
                let sdk_path = String::from_utf8_lossy(&output.stdout).trim().to_string();
                // Set the system library root to ensure the linker finds all SDK libraries
                println!("cargo::rustc-link-arg=-Wl,-syslibroot,{sdk_path}");
            }
        }

        // Explicitly link the ObjC runtime library for objc_get_class, etc.
        println!("cargo::rustc-link-arg=-lobjc");
    }
}
