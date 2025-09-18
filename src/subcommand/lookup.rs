use super::*;

#[derive(Debug, Parser)]
pub struct Lookup {
  #[clap(short, long)]
  name: String,
  #[clap(short, long, help = "Maximum number of items to return")]
  limit: Option<usize>,
  #[clap(short, long, help = "Number of items to skip for pagination")]
  offset: Option<usize>,
  #[clap(
    short = 't',
    long,
    help = "Filter by item type: function, struct, enum, trait, macro, type, constant, module"
  )]
  item_type: Option<String>,
  #[clap(
    short,
    long,
    help = "Search term to filter items by name or description"
  )]
  query: Option<String>,
}

impl Into<LookupCrateRequest> for Lookup {
  fn into(self) -> LookupCrateRequest {
    LookupCrateRequest {
      name: self.name,
      limit: self.limit,
      offset: self.offset,
      item_type: self.item_type,
      query: self.query,
    }
  }
}

impl Lookup {
  pub async fn run(self) -> Result {
    let documentation = lookup_crate(&self.into())?;
    println!("{}", serde_json::to_string(&documentation)?.trim());
    Ok(())
  }
}
