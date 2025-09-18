use super::*;

pub async fn run() -> Result {
  let crates = list_crates()?.join("\n");
  println!("{crates}");
  Ok(())
}
