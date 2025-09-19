use super::*;

const DOC_PATH: &str = "target/doc";

pub fn generate_docs(request: &GenerateDocsRequest) -> Result<String> {
  let mut cmd = Command::new("cargo");

  cmd.arg("doc");

  if let Some(ref flags) = request.flags {
    for flag in flags {
      cmd.arg(flag);
    }
  }

  let output = cmd
    .output()
    .map_err(|e| anyhow!("failed to run cargo doc: {}", e))?;

  let stderr = String::from_utf8_lossy(&output.stderr);

  if !output.status.success() {
    bail!("cargo doc failed: {}", stderr);
  }

  let stdout = String::from_utf8_lossy(&output.stdout);

  let mut result = String::new();

  if !stdout.is_empty() {
    result.push_str("STDOUT:\n");
    result.push_str(&stdout);
  }

  if !stderr.is_empty() {
    if !result.is_empty() {
      result.push_str("\n\n");
    }

    result.push_str("STDERR:\n");

    result.push_str(&stderr);
  }

  if result.is_empty() {
    result = "Documentation generated successfully.".to_string();
  }

  Ok(result)
}

pub fn list_crates() -> Result<Vec<String>> {
  list_crates_in_path(DOC_PATH)
}

pub fn lookup_crate(request: &LookupCrateRequest) -> Result<Documentation> {
  lookup_crate_in_path(request, DOC_PATH)
}
