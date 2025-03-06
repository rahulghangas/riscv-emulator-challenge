use std::env;

fn main() {
    // Only enable SIMD features on x86_64 architecture
    if env::var("TARGET").unwrap_or_default().contains("x86_64") {
        // Enable configuration for SIMD instruction sets
        println!("cargo:rustc-cfg=target_feature=\"sse2\"");
        println!("cargo:rustc-cfg=target_feature=\"avx2\"");
    }
    
    // Rerun the build script if it changes
    println!("cargo:rerun-if-changed=build.rs");
} 