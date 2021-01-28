static ACCESS_TOKEN: &'static str = include_str!("secrets");

fn main() {
    println!("cargo:rustc-env=ACCESS_TOKEN={}", ACCESS_TOKEN);
}
