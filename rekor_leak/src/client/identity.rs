use std::collections::BTreeMap;
use x509_parser::extensions::GeneralName;
use x509_parser::prelude::*;

const ENFORCE_DEV_ISSUER: &str = "https://issuer.enforce.dev";

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct RepoOwner {
    pub slug: String,
    pub uri: Option<String>,
}

impl RepoOwner {
    #[cfg(test)]
    pub fn slug(slug: impl Into<String>) -> Self {
        Self {
            slug: slug.into(),
            uri: None,
        }
    }
}

impl TryFrom<&str> for RepoOwner {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let (path, uri_prefix) = match value.strip_prefix("https://github.com/") {
            Some(path) => (path, Some("https://github.com/")),
            // Don't treat the host of an unsupported URL as an owner.
            None if value.contains("://") => return Err(()),
            None => (value, None),
        };

        let slug = path.split('/').next().unwrap_or_default();
        if slug.is_empty() {
            return Err(());
        }

        Ok(Self {
            slug: slug.to_owned(),
            uri: uri_prefix.map(|prefix| format!("{prefix}{slug}")),
        })
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FulcioIdentity {
    /// Extracted organization (from `source_repo_owner`, or parsed from URI)
    pub(super) organization: Option<RepoOwner>,
    pub(super) email: Option<String>,
    pub(super) subject_uri: Option<String>,
    pub(super) oidc_issuer_v1: Option<String>,
    pub(super) oidc_issuer_v2: Option<String>,
    pub(super) workflow_trigger: Option<String>,
    pub(super) workflow_sha: Option<String>,
    pub(super) workflow_name: Option<String>,
    pub(super) workflow_repository: Option<String>,
    pub(super) workflow_ref: Option<String>,
    pub(super) build_signer_uri: Option<String>,
    pub(super) build_signer_digest: Option<String>,
    pub(super) runner_environment: Option<String>,
    pub(super) source_repo_uri: Option<String>,
    pub(super) source_repo_digest: Option<String>,
    pub(super) source_repo_ref: Option<String>,
    pub(super) source_repo_identifier: Option<String>,
    pub(super) source_repo_owner: Option<String>,
    pub(super) source_repo_owner_id: Option<String>,
    pub(super) build_config_uri: Option<String>,
    pub(super) build_config_digest: Option<String>,
    pub(super) build_trigger: Option<String>,
    pub(super) run_invocation_uri: Option<String>,
    pub(super) source_repo_visibility: Option<String>,
    pub(super) extras: BTreeMap<String, String>, // anything else under the arc
}

impl FulcioIdentity {
    /// Build from a DER-encoded X.509 certificate; `None` if unparseable.
    pub fn from_der(der: &[u8]) -> Option<Self> {
        let (_, cert) = X509Certificate::from_der(der).ok()?;
        Some(cert.into())
    }

    pub fn is_actionable(&self) -> bool {
        let issuer = self
            .oidc_issuer_v2
            .as_deref()
            .or(self.oidc_issuer_v1.as_deref());

        if issuer == Some(ENFORCE_DEV_ISSUER) {
            return false;
        }

        self.organization.is_some() || self.email.is_some()
    }
}

impl<'a> From<X509Certificate<'a>> for FulcioIdentity {
    fn from(crt: X509Certificate<'a>) -> Self {
        let mut out = Self::default();

        // Extract SAN (Subject Alternative Names) for email and URI
        if let Ok(Some(san_ext)) = crt.subject_alternative_name() {
            for name in &san_ext.value.general_names {
                match name {
                    GeneralName::RFC822Name(email) => {
                        out.email = Some(email.to_string());
                    }
                    GeneralName::URI(uri) => {
                        out.subject_uri = Some(uri.to_string());
                    }
                    _ => {}
                }
            }
        }

        // Extract custom Sigstore/Fulcio OID extensions
        for ext in crt.extensions() {
            let oid = ext.oid.to_id_string();

            let Some(val) = decode_fulcio_value(ext.value) else {
                continue;
            };
            match oid.as_str() {
                "1.3.6.1.4.1.57264.1.1" => out.oidc_issuer_v1 = Some(val),
                "1.3.6.1.4.1.57264.1.2" => out.workflow_trigger = Some(val),
                "1.3.6.1.4.1.57264.1.3" => out.workflow_sha = Some(val),
                "1.3.6.1.4.1.57264.1.4" => out.workflow_name = Some(val),
                "1.3.6.1.4.1.57264.1.5" => out.workflow_repository = Some(val),
                "1.3.6.1.4.1.57264.1.6" => out.workflow_ref = Some(val),
                "1.3.6.1.4.1.57264.1.8" => out.oidc_issuer_v2 = Some(val),
                "1.3.6.1.4.1.57264.1.9" => out.build_signer_uri = Some(val),
                "1.3.6.1.4.1.57264.1.10" => out.build_signer_digest = Some(val),
                "1.3.6.1.4.1.57264.1.11" => out.runner_environment = Some(val),
                "1.3.6.1.4.1.57264.1.12" => out.source_repo_uri = Some(val),
                "1.3.6.1.4.1.57264.1.13" => out.source_repo_digest = Some(val),
                "1.3.6.1.4.1.57264.1.14" => out.source_repo_ref = Some(val),
                "1.3.6.1.4.1.57264.1.15" => out.source_repo_identifier = Some(val),
                "1.3.6.1.4.1.57264.1.16" => out.source_repo_owner = Some(val),
                "1.3.6.1.4.1.57264.1.17" => out.source_repo_owner_id = Some(val),
                "1.3.6.1.4.1.57264.1.18" => out.build_config_uri = Some(val),
                "1.3.6.1.4.1.57264.1.19" => out.build_config_digest = Some(val),
                "1.3.6.1.4.1.57264.1.20" => out.build_trigger = Some(val),
                "1.3.6.1.4.1.57264.1.21" => out.run_invocation_uri = Some(val),
                "1.3.6.1.4.1.57264.1.22" => out.source_repo_visibility = Some(val),
                other => {
                    // Ignore standard X.509 OIDs (2.5.29.*) in extras
                    if other.starts_with("1.3.6.1.4.1.57264.1.") {
                        out.extras.insert(other.to_owned(), val);
                    }
                }
            }
        }
        out.organization = repo_owner(out.source_repo_owner.as_deref())
            .or_else(|| repo_owner(out.source_repo_uri.as_deref()))
            .or_else(|| repo_owner(out.workflow_repository.as_deref()));
        out
    }
}

fn repo_owner(value: Option<&str>) -> Option<RepoOwner> {
    value.and_then(|value| RepoOwner::try_from(value).ok())
}

fn decode_fulcio_value(raw: &[u8]) -> Option<String> {
    if let Ok((_, s)) = asn1_rs::Utf8String::from_der(raw) {
        return Some(s.string().to_owned());
    }
    std::str::from_utf8(raw).ok().map(str::to_owned)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn repo_owner_from_github_uri() {
        let owner = RepoOwner::try_from("https://github.com/sigstore/cosign").unwrap();
        assert_eq!(owner.slug, "sigstore");
        assert_eq!(owner.uri.as_deref(), Some("https://github.com/sigstore"));

        let owner = RepoOwner::try_from("https://github.com/sigstore").unwrap();
        assert_eq!(owner.slug, "sigstore");
        assert_eq!(owner.uri.as_deref(), Some("https://github.com/sigstore"));

        assert!(RepoOwner::try_from("https://gitlab.com/foo/bar").is_err());
        assert!(RepoOwner::try_from("").is_err());
    }

    #[test]
    fn repo_owner_from_repository_slug() {
        let owner = RepoOwner::try_from("sigstore/cosign").unwrap();
        assert_eq!(owner.slug, "sigstore");
        assert_eq!(owner.uri, None);

        let owner = RepoOwner::try_from("sigstore").unwrap();
        assert_eq!(owner.slug, "sigstore");
        assert_eq!(owner.uri, None);

        assert!(RepoOwner::try_from("").is_err());
    }

    #[test]
    fn enforce_dev_not_actionable() {
        let id = FulcioIdentity {
            oidc_issuer_v2: Some("https://issuer.enforce.dev".to_owned()),
            organization: Some(RepoOwner::slug("chainguard")),
            ..Default::default()
        };
        assert!(!id.is_actionable());
    }

    #[test]
    fn identity_with_org_is_actionable() {
        let id = FulcioIdentity {
            organization: Some(RepoOwner::slug("sigstore")),
            ..Default::default()
        };
        assert!(id.is_actionable());
    }

    #[test]
    fn identity_with_email_only_is_actionable() {
        let id = FulcioIdentity {
            email: Some("user@example.com".to_owned()),
            ..Default::default()
        };
        assert!(id.is_actionable());
    }

    #[test]
    fn empty_identity_not_actionable() {
        let id = FulcioIdentity::default();
        assert!(!id.is_actionable());
    }
}
