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

impl From<Lookup> for LookupCrateRequest {
  fn from(value: Lookup) -> Self {
    LookupCrateRequest {
      name: value.name,
      limit: value.limit,
      offset: value.offset,
      item_type: value.item_type,
      query: value.query,
    }
  }
}

impl Lookup {
  pub async fn run(self) -> Result {
    let documentation = lookup_crate(&self.into(), DOC_PATH)?;
    println!("{}", serde_json::to_string(&documentation)?.trim());
    Ok(())
  }
}
