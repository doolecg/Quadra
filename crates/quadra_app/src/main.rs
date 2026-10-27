// Entry point. The window and viewport arrive in later M1 steps; for now this
// proves the workspace builds and runs.

const NOTICE: &str = "Provided \"as is\", without warranty, under the Apache 2.0 licence. \
To the extent permitted by law, the authors are not liable for any loss or damage from using it. \
Back up your work.";

fn main() {
    println!("Quadra {}", env!("CARGO_PKG_VERSION"));
    println!("{NOTICE}");
}
