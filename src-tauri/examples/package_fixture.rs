#[cfg(debug_assertions)]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let root = std::env::args_os()
        .nth(1)
        .ok_or("new fixture workspace required")?;
    println!(
        "{}",
        researchclip_lib::qa_packages::seed(std::path::Path::new(&root))?
    );
    Ok(())
}
#[cfg(not(debug_assertions))]
fn main() {
    eprintln!("Fixture tooling is debug-only");
    std::process::exit(1);
}
