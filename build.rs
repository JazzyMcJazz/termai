fn main() {
    let cargo_toml = std::fs::read_to_string("Cargo.toml").expect("Failed to read Cargo.toml");

    let release_date = cargo_toml
        .lines()
        .find(|line| line.starts_with("release_date"))
        .expect("Failed to find release_date in Cargo.toml")
        .split('=')
        .last()
        .expect("Failed to parse release_date in Cargo.toml")
        .trim()
        .trim_matches('"');

    println!("cargo:rustc-env=RELEASE_DATE={}", release_date);
}
