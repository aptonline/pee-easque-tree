use crate::types::{FetchResult, PackageInfo, PS3UpdateError, Result};
use crate::utils::{clean_title_id, format_size};
use quick_xml::de::from_str;
use serde::Deserialize;

const PS3_UPDATE_BASE_URL: &str = "https://a0.ww.np.dl.playstation.net";

/// XML structure for parsing Sony's update XML
#[derive(Debug, Deserialize)]
struct PackageAttr {
    #[serde(rename = "@url")]
    url: Option<String>,
    #[serde(rename = "@digest")]
    digest: Option<String>,
    #[serde(rename = "@sha1")]
    sha1: Option<String>,
    #[serde(rename = "@size")]
    size: Option<String>,
    #[serde(rename = "@version")]
    version: Option<String>,
    #[serde(rename = "@ps3_system_ver")]
    ps3_system_ver: Option<String>,
    #[serde(rename = "PARAMSFO")]
    paramsfo: Option<ParamsFo>,
}

#[derive(Debug, Deserialize)]
struct ParamsFo {
    #[serde(rename = "TITLE")]
    title: Option<String>,
}

#[derive(Debug, Deserialize)]
struct TitlePatch {
    #[serde(rename = "package")]
    package: Option<Vec<PackageAttr>>,
    #[serde(rename = "PACKAGE")]
    PACKAGE: Option<Vec<PackageAttr>>,
    #[serde(rename = "tag")]
    tag: Option<TagNode>,
    #[serde(rename = "TAG")]
    TAG: Option<TagNode>,
}

#[derive(Debug, Deserialize)]
struct TagNode {
    #[serde(rename = "package")]
    package: Option<Vec<PackageAttr>>,
    #[serde(rename = "PACKAGE")]
    PACKAGE: Option<Vec<PackageAttr>>,
}

/// PS3 Update Fetcher
pub struct UpdateFetcher {
    client: reqwest::Client,
}

impl UpdateFetcher {
    /// Create a new UpdateFetcher with default settings
    pub fn new() -> Result<Self> {
        let client = reqwest::Client::builder()
            .danger_accept_invalid_certs(true)
            .build()?;
        Ok(Self { client })
    }

    /// Check if the PS3 update server is accessible
    pub async fn check_server_status(&self) -> bool {
        self.client
            .head(PS3_UPDATE_BASE_URL)
            .send()
            .await
            .is_ok()
    }

    /// Fetch available updates for a given PS3 title ID
    pub async fn fetch_updates(&self, title_id: &str) -> Result<FetchResult> {
        let cleaned = clean_title_id(title_id);

        if cleaned.is_empty() {
            return Err(PS3UpdateError::InvalidTitleId(
                "Empty or invalid Title ID".into(),
            ));
        }

        let url = format!(
            "{}/tpl/np/{id}/{id}-ver.xml",
            PS3_UPDATE_BASE_URL,
            id = cleaned
        );

        let resp = self.client.get(&url).send().await?;

        if !resp.status().is_success() {
            return Err(PS3UpdateError::NoUpdatesFound(cleaned));
        }

        let text = resp.text().await?;

        // Try to extract <TITLE> directly from raw XML as a fallback
        let raw_title = Self::extract_title_from_xml(&text);

        let parsed: TitlePatch = from_str(&text)
            .map_err(|e| PS3UpdateError::XmlParse(e.to_string()))?;

        let game_title = raw_title.unwrap_or_else(|| "Unknown Title".to_string());
        let pkgs = Self::extract_packages(parsed);

        if pkgs.is_empty() {
            return Ok(FetchResult {
                results: vec![],
                error: Some(format!("No <package> entries found in XML for {}", cleaned)),
                game_title,
                cleaned_title_id: cleaned,
            });
        }

        // Override game title if available in package metadata
        let game_title = pkgs
            .get(0)
            .and_then(|p| p.paramsfo.as_ref())
            .and_then(|pf| pf.title.as_ref())
            .map(|t| t.trim().to_string())
            .unwrap_or(game_title);

        let mut results: Vec<PackageInfo> = pkgs
            .into_iter()
            .map(|p| Self::package_attr_to_info(p))
            .collect();

        // Sort by version (highest first)
        results.sort_by(|a, b| {
            let va = a.version.parse::<f32>().unwrap_or(0.0);
            let vb = b.version.parse::<f32>().unwrap_or(0.0);
            vb.partial_cmp(&va).unwrap_or(std::cmp::Ordering::Equal)
        });

        Ok(FetchResult {
            results,
            error: None,
            game_title,
            cleaned_title_id: cleaned,
        })
    }

    fn extract_title_from_xml(text: &str) -> Option<String> {
        if let Some(start) = text.find("<TITLE>") {
            if let Some(end) = text[start + 7..].find("</TITLE>") {
                let t = &text[start + 7..start + 7 + end];
                let cleaned = t.trim().to_string();
                if !cleaned.is_empty() {
                    return Some(cleaned);
                }
            }
        }
        None
    }

    fn extract_packages(tp: TitlePatch) -> Vec<PackageAttr> {
        let mut pkgs: Vec<PackageAttr> = vec![];

        if let Some(tag) = tp.tag.or(tp.TAG) {
            if let Some(mut list) = tag.package {
                pkgs.append(&mut list);
            }
            if let Some(mut list) = tag.PACKAGE {
                pkgs.append(&mut list);
            }
        }

        if pkgs.is_empty() {
            if let Some(mut list) = tp.package {
                pkgs.append(&mut list);
            }
            if let Some(mut list) = tp.PACKAGE {
                pkgs.append(&mut list);
            }
        }

        pkgs
    }

    fn package_attr_to_info(p: PackageAttr) -> PackageInfo {
        let mut url = p.url.unwrap_or_default();
        url = url.trim().to_string();

        let digest = p
            .digest
            .or(p.sha1)
            .unwrap_or_default()
            .trim()
            .to_string();

        let version = p
            .version
            .unwrap_or_else(|| "Unknown".into())
            .to_string();

        let system_ver = p.ps3_system_ver.unwrap_or_default().to_string();

        let size_bytes: u64 = p
            .size
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(0);

        let filename = url
            .split('/')
            .last()
            .unwrap_or("update.pkg")
            .to_string();

        PackageInfo {
            version,
            system_ver,
            size_bytes,
            size_human: format_size(size_bytes),
            url,
            sha1: digest,
            filename,
        }
    }
}

impl Default for UpdateFetcher {
    fn default() -> Self {
        Self::new().expect("Failed to create UpdateFetcher")
    }
}
