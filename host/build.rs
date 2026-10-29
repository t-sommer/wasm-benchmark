fn main() {
    let workspace = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..");
    let guest_target = workspace.join("target-guest");
    let core = guest_target.join("wasm32-unknown-unknown/release/guest_core.wasm");

    println!("cargo:rerun-if-changed={}", workspace.join("guest-core").display());
    println!("cargo:rerun-if-changed={}", workspace.join("guest-component").display());
    println!("cargo:rerun-if-changed={}", workspace.join("wit").display());
    println!("cargo:rerun-if-changed={}", workspace.join("scripts/build-guests.ps1").display());
    println!("cargo:rerun-if-changed={}", workspace.join("scripts/build-guests.sh").display());

    if core.exists() {
        return;
    }

    println!("cargo:warning=Guest wasm not built yet. Run scripts/build-guests.ps1 (or .sh), then re-run cargo bench -p host.");
}
