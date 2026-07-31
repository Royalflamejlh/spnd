//! crates.io pointer for the spnd CLI.
//!
//! The full application lives in a Cargo workspace with internal path
//! dependencies, so this crate ships install directions instead of the
//! binary itself.
fn main() {
    eprintln!("This crates.io package is a pointer; the full spnd binary installs with:");
    eprintln!();
    eprintln!("  cargo install --git https://github.com/Royalflamejlh/spnd spnd");
    eprintln!("  npm install -g @spnd/spnd");
    eprintln!(
        "  curl -fsSL https://raw.githubusercontent.com/Royalflamejlh/spnd/main/install.sh | sh"
    );
    eprintln!();
    eprintln!("Prebuilt binaries: https://github.com/Royalflamejlh/spnd/releases");
    std::process::exit(1);
}
