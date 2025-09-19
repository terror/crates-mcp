use super::*;

const DOC_PATH: &str = "target/doc";

pub async fn run() -> Result {
  let crates = list_crates(DOC_PATH)?.join("\n");
  println!("{crates}");
  Ok(())
}
