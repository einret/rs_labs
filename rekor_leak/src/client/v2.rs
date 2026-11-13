use async_trait::async_trait;
use sigstore_rekor::body::RekorEntryBody;
use sigstore_rekor::client::RekorV2Client;

use super::error::RekorApiError;
use super::http::RekorApi;
use super::identity::FulcioIdentity;
use super::result::LeakResult;

const TILE_WIDTH: u64 = 256;

#[async_trait]
impl RekorApi for RekorV2Client {
    async fn get_entry_by_index(&self, index: u64) -> Result<LeakResult, RekorApiError> {
        self.get_tile_containing(index)
            .await?
            .into_iter()
            .find(|r| r.log_index == index)
            .ok_or_else(|| {
                RekorApiError::UnsupportedEntry(format!("v2: entry not found at index {index}"))
            })
    }

    async fn get_tile_containing(&self, log_index: u64) -> Result<Vec<LeakResult>, RekorApiError> {
        // C2SP tlog-tiles: `tile/entries/N` holds entries N*256..=N*256+255.
        let tile_index = log_index / TILE_WIDTH;
        let bundle = self
            .get_entry_bundle(tile_index, None)
            .await
            .map_err(RekorApiError::from)?;

        tokio::task::spawn_blocking(move || {
            let jsons = decode_bundle(&bundle.bytes)?;
            let first_index = bundle.index * TILE_WIDTH;
            let mut results = Vec::with_capacity(jsons.len());
            for (offset, json) in jsons.iter().enumerate() {
                let idx = first_index + offset as u64;
                match parse_v2_entry(idx, json.as_bytes()) {
                    Ok(result) => results.push(result),
                    Err(error) => eprintln!("[v2] skipping log index {idx}: {error}"),
                }
            }
            Ok(results)
        })
        .await
        .map_err(|e| RekorApiError::UnsupportedEntry(format!("v2: blocking task join: {e}")))?
    }

    fn scan_unit_width(&self) -> u64 {
        TILE_WIDTH
    }
}

// documentation of the structure of a tile https://github.com/C2SP/C2SP/blob/main/tlog-tiles.md
fn decode_bundle(bytes: &[u8]) -> Result<Vec<String>, RekorApiError> {
    let mut entries = Vec::new();
    let mut rest = bytes;

    while rest.len() >= 2 {
        let len = u16::from_be_bytes([rest[0], rest[1]]) as usize;
        rest = &rest[2..];

        let record = rest
            .get(..len)
            .ok_or_else(|| RekorApiError::UnsupportedEntry("v2: truncated bundle record".into()))?;
        rest = &rest[len..];

        let json = std::str::from_utf8(record).map_err(|e| {
            RekorApiError::UnsupportedEntry(format!("v2: entry is not UTF-8 JSON: {e}"))
        })?;
        entries.push(json.to_owned());
    }

    if !rest.is_empty() {
        return Err(RekorApiError::UnsupportedEntry(
            "v2: truncated bundle length prefix".into(),
        ));
    }

    if entries.is_empty() {
        return Err(RekorApiError::UnsupportedEntry(
            "v2: entry bundle was empty".into(),
        ));
    }

    Ok(entries)
}

pub(crate) fn parse_v2_entry(index: u64, tle: &[u8]) -> Result<LeakResult, RekorApiError> {
    let v: serde_json::Value = serde_json::from_slice(tle)?;
    let kind = v["kind"].as_str().unwrap_or_default();
    let version = v["apiVersion"].as_str().unwrap_or_default();

    let body = RekorEntryBody::from_json(tle, kind, version)?;

    let mut identities = Vec::<FulcioIdentity>::new();

    match body {
        RekorEntryBody::HashedRekordV002(eb) => {
            let verifier = &eb.spec.hashed_rekord_v002.signature.verifier;
            if let Some(id) = verifier
                .x509_certificate
                .as_ref()
                .map(|cert| cert.raw_bytes.as_ref())
                .and_then(FulcioIdentity::from_der)
            {
                identities.push(id);
            }
        }
        RekorEntryBody::DsseV002(eb) => {
            for sig in &eb.spec.dsse_v002.signatures {
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
                "v2: unsupported entry kind/version: {kind}/{version}"
            )));
        }
    }

    Ok(LeakResult {
        log_index: index,
        api_version: version.to_string(),
        kind: kind.to_string(),
        identities,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn c2sp_tile_math() {
        assert_eq!(200_000 / TILE_WIDTH, 781);
        assert_eq!(781 * TILE_WIDTH, 199_936);
        assert_eq!(253_696 / TILE_WIDTH, 991);
        assert_eq!(290_304 / TILE_WIDTH, 1_134);
    }

    #[test]
    fn decode_bundle_reads_length_prefixed_records() {
        let mut bytes = Vec::new();
        for record in [r#"{"a":1}"#, r#"{"b":2}"#] {
            bytes.extend_from_slice(&(record.len() as u16).to_be_bytes());
            bytes.extend_from_slice(record.as_bytes());
        }

        assert_eq!(
            decode_bundle(&bytes).unwrap(),
            vec![r#"{"a":1}"#.to_owned(), r#"{"b":2}"#.to_owned()]
        );
    }

    #[test]
    fn decode_bundle_rejects_trailing_byte() {
        let bytes = [0u8, 2, b'{', b'}', 0];
        assert!(decode_bundle(&bytes).is_err());
    }

    #[test]
    fn unsupported_variant_is_an_error() {
        let tle = br#"{
            "kind": "hashedrekord",
            "apiVersion": "0.0.1",
            "spec": {
                "data": { "hash": { "algorithm": "sha256", "value": "00" } },
                "signature": { "content": "AA==", "publicKey": { "content": "AA==" } }
            }
        }"#;

        assert!(matches!(
            parse_v2_entry(0, tle),
            Err(RekorApiError::UnsupportedEntry(_))
        ));
    }
}
