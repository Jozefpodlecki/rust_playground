
#[cfg(all(target_os = "windows"))]
fn main() {
    let mut resource = winres::WindowsResource::new();
    resource.set_manifest_file("app.manifest");

    if let Err(error) = resource.compile() {
        eprint!("{error}");
        std::process::exit(1);
    }
}