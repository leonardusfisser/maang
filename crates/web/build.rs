fn main() {
    // Placeholder for build-time tasks (static file embedding, etc.)
    println!("cargo:rerun-if-changed=static/");
    println!("cargo:rerun-if-changed=templates_shared/");
    println!("cargo:rerun-if-changed=templates_tenants/");
    println!("cargo:rerun-if-changed=templates_components/");
    println!("cargo:rerun-if-changed=templates_webmaster/");
}