use anyhow::Result;
use serde::Deserialize;

#[derive(Debug, Clone)]
pub struct PackageInfo {
    pub name: String,
    pub version: String,
    #[allow(dead_code)]
    pub description: Option<String>,
    #[allow(dead_code)]
    pub downloads: Option<u64>,
}

// ─────────────────────────────────────────────
// npm registry types
// ─────────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct NpmPackageResponse {
    name: String,
    #[serde(rename = "dist-tags", default)]
    dist_tags: Option<NpmDistTags>,
    description: Option<String>,
}

#[derive(Debug, Deserialize)]
struct NpmDistTags {
    latest: Option<String>,
}

// ─────────────────────────────────────────────
// crates.io types
// ─────────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct CratesIoResponse {
    #[serde(rename = "crate")]
    krate: CratesIoCrate,
}

#[derive(Debug, Deserialize)]
struct CratesIoCrate {
    name: String,
    max_version: String,
    description: Option<String>,
    downloads: u64,
}

// ─────────────────────────────────────────────
// PyPI types
// ─────────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct PyPIResponse {
    info: PyPIInfo,
}

#[derive(Debug, Deserialize)]
struct PyPIInfo {
    name: String,
    version: String,
    summary: Option<String>,
}

// ─────────────────────────────────────────────
// Main search dispatcher
// ─────────────────────────────────────────────

pub fn search_package(registry: &str, package_name: &str) -> Result<PackageInfo> {
    let normalized_registry = registry.to_lowercase();
    match normalized_registry.as_str() {
        "npm" | "npmjs" | "npmjs.com" | "node" | "javascript" => search_npm(package_name),
        "crates.io" | "crates" | "cargo" | "rust" => search_crates_io(package_name),
        "pypi" | "pip" | "python" => search_pypi(package_name),
        _ => {
            anyhow::bail!(
                "Unsupported registry: '{}'. Supported: npm, crates.io, pypi",
                registry
            )
        }
    }
}

// ─────────────────────────────────────────────
// npm search
// ─────────────────────────────────────────────

fn search_npm(package_name: &str) -> Result<PackageInfo> {
    let url = format!("https://registry.npmjs.org/{}", package_name);

    let client = reqwest::blocking::Client::builder()
        .user_agent("prompt-generator-tui/1.0")
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .map_err(|e| anyhow::anyhow!("Failed to create HTTP client: {}", e))?;

    let response = client
        .get(&url)
        .send()
        .map_err(|e| anyhow::anyhow!("Network error searching npm: {}", e))?;

    if response.status() == 404 {
        anyhow::bail!("Package '{}' not found on npm", package_name);
    }

    if !response.status().is_success() {
        anyhow::bail!(
            "npm registry returned status {} for '{}'",
            response.status(),
            package_name
        );
    }

    let npm_response: NpmPackageResponse = response
        .json()
        .map_err(|e| anyhow::anyhow!("Failed to parse npm response: {}", e))?;

    let version = npm_response
        .dist_tags
        .and_then(|dt| dt.latest)
        .unwrap_or_else(|| "unknown".to_string());

    Ok(PackageInfo {
        name: npm_response.name,
        version,
        description: npm_response.description,
        downloads: None,
    })
}

// ─────────────────────────────────────────────
// crates.io search
// ─────────────────────────────────────────────

fn search_crates_io(package_name: &str) -> Result<PackageInfo> {
    let url = format!("https://crates.io/api/v1/crates/{}", package_name);

    let client = reqwest::blocking::Client::builder()
        .user_agent("prompt-generator-tui/1.0")
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .map_err(|e| anyhow::anyhow!("Failed to create HTTP client: {}", e))?;

    let response = client
        .get(&url)
        .send()
        .map_err(|e| anyhow::anyhow!("Network error searching crates.io: {}", e))?;

    if response.status() == 404 {
        anyhow::bail!("Crate '{}' not found on crates.io", package_name);
    }

    if !response.status().is_success() {
        anyhow::bail!(
            "crates.io returned status {} for '{}'",
            response.status(),
            package_name
        );
    }

    let crates_response: CratesIoResponse = response
        .json()
        .map_err(|e| anyhow::anyhow!("Failed to parse crates.io response: {}", e))?;

    Ok(PackageInfo {
        name: crates_response.krate.name,
        version: crates_response.krate.max_version,
        description: crates_response.krate.description,
        downloads: Some(crates_response.krate.downloads),
    })
}

// ─────────────────────────────────────────────
// PyPI search
// ─────────────────────────────────────────────

fn search_pypi(package_name: &str) -> Result<PackageInfo> {
    let url = format!("https://pypi.org/pypi/{}/json", package_name);

    let client = reqwest::blocking::Client::builder()
        .user_agent("prompt-generator-tui/1.0")
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .map_err(|e| anyhow::anyhow!("Failed to create HTTP client: {}", e))?;

    let response = client
        .get(&url)
        .send()
        .map_err(|e| anyhow::anyhow!("Network error searching PyPI: {}", e))?;

    if response.status() == 404 {
        anyhow::bail!("Package '{}' not found on PyPI", package_name);
    }

    if !response.status().is_success() {
        anyhow::bail!(
            "PyPI returned status {} for '{}'",
            response.status(),
            package_name
        );
    }

    let pypi_response: PyPIResponse = response
        .json()
        .map_err(|e| anyhow::anyhow!("Failed to parse PyPI response: {}", e))?;

    Ok(PackageInfo {
        name: pypi_response.info.name,
        version: pypi_response.info.version,
        description: pypi_response.info.summary,
        downloads: None,
    })
}
