use {super::*, lookup::Lookup};

mod list;
mod lookup;
mod server;

#[derive(Debug, Parser)]
pub enum Subcommand {
  List,
  Lookup(Lookup),
  Server,
}

impl Subcommand {
  pub async fn run(self) -> Result {
    match self {
      Self::List => list::run().await,
      Self::Lookup(lookup) => lookup.run().await,
      Self::Server => server::run().await,
    }
  }
}
