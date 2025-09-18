use super::*;

#[derive(Debug, Parser)]
pub struct Lookup {
  #[clap(short, long)]
  name: String,
}

impl Lookup {
  pub async fn run(self) -> Result {
    let documentation = lookup_crate(&self.name)?;
    println!("{}", serde_yaml::to_string(&documentation)?);
    Ok(())
  }
}
