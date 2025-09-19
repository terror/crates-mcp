use super::*;

const DOC_PATH: &str = "target/doc";

pub fn generate_docs(request: &GenerateDocsRequest) -> Result<String> {
  let output = Command::new("cargo")
    .arg("doc")
    .args(request.flags.as_deref().unwrap_or(&[]))
    .output()
    .map_err(|error| anyhow!("failed to run cargo doc: {}", error))?;

  let stderr = String::from_utf8_lossy(&output.stderr);

  if !output.status.success() {
    bail!("cargo doc failed: {}", stderr);
  }

  let stdout = String::from_utf8_lossy(&output.stdout);

  Ok(format!("{}{}", stdout, stderr))
}

pub fn list_crates() -> Result<Vec<String>> {
  list_crates_in_path(DOC_PATH)
}

pub fn lookup_crate(request: &LookupCrateRequest) -> Result<Documentation> {
  lookup_crate_in_path(request, DOC_PATH)
}
