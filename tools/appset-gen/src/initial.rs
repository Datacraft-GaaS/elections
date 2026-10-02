// Copyright (c) 2026 Nyimbi Odero
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use crate::model::all_canonical_services;
use anyhow::Result;

/// Write a template deploy/service.yaml for each of the 18 canonical services.
pub fn write_templates() -> Result<()> {
    let services = all_canonical_services();
    for name in &services {
        let dir = format!("services/{}/deploy", name);
        std::fs::create_dir_all(&dir)?;
        let path = format!("{}/service.yaml", dir);
        if std::path::Path::new(&path).exists() {
            eprintln!("  skip  {path} (exists)");
            continue;
        }
        let content = template(name);
        std::fs::write(&path, content)?;
        eprintln!("  wrote {path}");
    }
    eprintln!("Done — {} service templates written", services.len());
    Ok(())
}

fn template(name: &str) -> String {
    format!(
        r#"# deploy/service.yaml — filled by the service owner (WP-1.9 template)
# Fields: name, image, artifacts, sites, strategy, replicas
name: {name}
image: harbor.eel.internal/eel/{name}
artifacts: []
# sites: leave empty to deploy to all 7 sites (primary, mirror, edge-*)
sites: []
strategy:
  primary: blue-green
  mirror:  blue-green
  edge-nairobi:  rolling
  edge-mombasa:  rolling
  edge-kisumu:   rolling
  edge-eldoret:  rolling
  edge-garissa:  rolling
replicas:
  primary:       3
  mirror:        3
  edge-nairobi:  2
  edge-mombasa:  2
  edge-kisumu:   2
  edge-eldoret:  2
  edge-garissa:  2
"#,
        name = name,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn template_is_valid_yaml() {
        let raw = template("rts-svc");
        let val: serde_yaml::Value =
            serde_yaml::from_str(&raw).expect("template must be valid YAML");
        assert_eq!(val["name"].as_str().unwrap(), "rts-svc");
    }

    #[test]
    fn template_snapshot() {
        let raw = template("portal-svc");
        insta::assert_snapshot!(raw);
    }
}
