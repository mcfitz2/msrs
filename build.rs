// build.rs: Scrape and bundle product data at build time
use std::process::Command;
use std::fs;
use std::path::Path;

fn main() {
    // Only rerun if scraper or models change
    println!("cargo:rerun-if-changed=src/metalsupermarkets/scraper.rs");
    println!("cargo:rerun-if-changed=src/metalsupermarkets/models.rs");

    // Only run the scraping process if MSRS_SCRAPE=1 is set in the environment
    if std::env::var("MSRS_SCRAPE").ok().as_deref() == Some("1") {
        // Path to the gather binary
        let gather_bin = if cfg!(windows) {
            "target/debug/gather.exe"
        } else {
            "target/debug/gather"
        };

        // Build the gather binary if needed
        let status = Command::new("cargo")
            .args(["build", "--bin", "gather"])
            .status()
            .expect("Failed to build gather binary");
        assert!(status.success(), "Failed to build gather binary");

        // Run the gather binary to generate resources/products.json
        let status = Command::new(gather_bin)
            .status()
            .expect("Failed to run gather binary");
        assert!(status.success(), "Failed to run gather binary");
    }

    // Always copy the products.json file (assume it exists)
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let dest = Path::new(&out_dir).join("products.json");
    fs::copy("src/resources/products.json", &dest).expect("Failed to copy products.json to OUT_DIR");
}
