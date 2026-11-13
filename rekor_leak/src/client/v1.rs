use async_trait::async_trait;
use sigstore_rekor::body::RekorEntryBody;
use sigstore_rekor::client::RekorClient;
use sigstore_rekor::entry::LogEntry;
use sigstore_types::encoding::LogIndex;

use super::error::RekorApiError;
use super::http::RekorApi;
use super::identity::FulcioIdentity;
use super::result::LeakResult;

#[async_trait]
impl RekorApi for RekorClient {
    async fn get_entry_by_index(&self, index: u64) -> Result<LeakResult, RekorApiError> {
        let log_index = LogIndex::new(index).map_err(|_| RekorApiError::InvalidLogIndex(index))?;
        let entry = RekorClient::get_entry_by_index(self, log_index)
            .await
            .map_err(RekorApiError::from)?;

        parse_v1(index, &entry)
    }
}

fn parse_v1(index: u64, entry: &LogEntry) -> Result<LeakResult, RekorApiError> {
    let body_bytes = entry.body.as_bytes();
    let v: serde_json::Value = serde_json::from_slice(body_bytes)?;

    let api_version = v["apiVersion"].as_str().unwrap_or_default();
    let kind = v["kind"].as_str().unwrap_or_default();

    let mut identities = Vec::<FulcioIdentity>::new();

    let body = RekorEntryBody::from_json(body_bytes, kind, api_version)?;

    match body {
        RekorEntryBody::IntotoV002(eb) => {
            for sig in eb.spec.content.envelope.signatures {
                let Some(id) = sig
                    .parse_certificate()
                    .ok()
                    .and_then(|der| FulcioIdentity::from_der(der.as_bytes()))
                else {
                    continue;
                };
                identities.push(id);
            }
        }
        RekorEntryBody::DsseV002(eb) => {
            for sig in eb.spec.dsse_v002.signatures {
                // Public-key-only DSSE entries carry no repository identity.
                let Some(id) = sig
                    .verifier
                    .x509_certificate
                    .as_ref()
                    .map(|cert| cert.raw_bytes.as_ref())
                    .and_then(FulcioIdentity::from_der)
                else {
                    continue;
                };
                identities.push(id);
            }
        }
        _other => {
            return Err(RekorApiError::UnsupportedEntry(format!(
                "v1: unsupported entry kind/version: {kind}/{api_version}"
            )));
        }
    }

    Ok(LeakResult {
        log_index: index,
        api_version: api_version.to_string(),
        kind: kind.to_string(),
        identities,
    })
}
