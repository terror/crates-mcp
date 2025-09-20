use super::*;

pub async fn run() -> Result {
  let crates = list_crates(DOC_PATH)?.join("\n");
  println!("{crates}");
  Ok(())
}
