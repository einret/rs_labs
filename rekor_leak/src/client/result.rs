use std::collections::{BTreeMap, BTreeSet};

use super::identity::FulcioIdentity;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct LeakSummary {
    log_index: u64,
    kind: String,
    api_version: String,
}

#[derive(Debug, Clone)]
struct LeakGroup {
    identity: FulcioIdentity,
    repos: BTreeSet<String>,
    workflows: BTreeSet<String>,
    visibilities: BTreeSet<String>,
    entries: BTreeSet<LeakSummary>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum IdentityKey {
    Org(String),
    Email(String),
}

impl IdentityKey {
    fn of(id: &FulcioIdentity) -> Option<Self> {
        match (&id.organization, &id.email) {
            (Some(owner), _) => Some(Self::Org(owner.slug.clone())),
            (None, Some(email)) => Some(Self::Email(email.clone())),
            _ => None,
        }
    }

    fn as_str(&self) -> &str {
        match self {
            Self::Org(slug) | Self::Email(slug) => slug,
        }
    }
}

#[derive(Debug, Default)]
pub(crate) struct LeakStore {
    groups: BTreeMap<IdentityKey, LeakGroup>,
}

#[derive(Debug, Clone)]
pub(crate) struct LeakResult {
    pub(crate) log_index: u64,
    pub(crate) api_version: String,
    pub(crate) kind: String,
    pub(crate) identities: Vec<FulcioIdentity>,
}

impl LeakResult {
    pub(crate) fn is_actionable(&self) -> bool {
        self.identities.iter().any(|id| id.is_actionable())
    }
}

impl LeakStore {
    pub(crate) fn insert(&mut self, leak: LeakResult) {
        if !leak.is_actionable() {
            return;
        }
        let summary = LeakSummary {
            log_index: leak.log_index,
            kind: leak.kind,
            api_version: leak.api_version,
        };

        for id in leak.identities {
            let Some(key) = IdentityKey::of(&id) else {
                continue;
            };
            let group = self.groups.entry(key).or_insert_with(|| LeakGroup {
                identity: id.clone(),
                repos: BTreeSet::new(),
                workflows: BTreeSet::new(),
                visibilities: BTreeSet::new(),
                entries: BTreeSet::new(),
            });

            if let Some(repo) = id
                .source_repo_uri
                .as_deref()
                .or(id.workflow_repository.as_deref())
            {
                group.repos.insert(repo.to_owned());
            }
            if let Some(workflow) = id.workflow_name.as_deref() {
                group.workflows.insert(workflow.to_owned());
            }
            if let Some(visibility) = id.source_repo_visibility.as_deref() {
                group.visibilities.insert(visibility.to_owned());
            }
            group.entries.insert(summary.clone());
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct DisplayOptions {
    pub(crate) view_entries: bool,
    pub(crate) private_only: bool,
    pub(crate) json: bool,
}

impl LeakStore {
    pub(crate) fn display(&self, opts: &DisplayOptions) {
        if opts.json {
            display_json(self, opts);
            return;
        }

        let mut printed = 0;
        for (key, group) in &self.groups {
            if opts.private_only && !group.is_private() {
                continue;
            }
            printed += 1;

            let tag = match key {
                IdentityKey::Org(_) => "org",
                IdentityKey::Email(_) => "email",
            };
            println!(
                "═══ [{tag}] {} ({} entries) ═══",
                key.as_str(),
                group.entries.len()
            );

            display_group_header(key, group);

            // Collapse indices by (kind, api_version).
            let mut by_kind: BTreeMap<(&str, &str), Vec<u64>> = BTreeMap::new();
            for entry in &group.entries {
                by_kind
                    .entry((entry.kind.as_str(), entry.api_version.as_str()))
                    .or_default()
                    .push(entry.log_index);
            }

            for ((kind, version), mut indices) in by_kind {
                indices.sort_unstable();
                println!("  {kind}/{version}: {} entries", indices.len());

                if opts.view_entries {
                    for idx in indices {
                        println!("    - {idx}");
                    }
                }
            }
            println!();
        }

        if printed == 0 {
            println!("No actionable leaks found.");
        }
    }
}

fn display_group_header(key: &IdentityKey, group: &LeakGroup) {
    let id = &group.identity;

    match key {
        IdentityKey::Org(_) => {
            if let Some(email) = &id.email {
                println!("  Email:        {email}");
            }
        }
        IdentityKey::Email(_) => {
            if let Some(owner) = &id.organization {
                println!("  Organization: {}", owner.slug);
                println!(
                    "  Repo Owner:   {}",
                    owner.uri.as_deref().unwrap_or(&owner.slug)
                );
            }
        }
    }

    for repo in &group.repos {
        println!("  Repository:   {repo}");
    }
    for workflow in &group.workflows {
        println!("  Workflow:     {workflow}");
    }
    if let Some(env) = &id.runner_environment {
        println!("  Runner Env:   {env}");
    }
    for visibility in &group.visibilities {
        println!("  Visibility:   {visibility}");
    }
    if let Some(issuer) = id
        .oidc_issuer_v2
        .as_deref()
        .or(id.oidc_issuer_v1.as_deref())
    {
        println!("  OIDC Issuer:  {issuer}");
    }
}

fn display_json(store: &LeakStore, opts: &DisplayOptions) {
    let groups: Vec<_> = store
        .groups
        .iter()
        .filter(|(_, group)| !opts.private_only || group.is_private())
        .map(|(key, group)| {
            let id = &group.identity;
            serde_json::json!({
                "key": key.as_str(),
                "organization": id.organization.as_ref().map(|owner| owner.slug.as_str()),
                "email": id.email.as_deref(),
                "repos": group.repos.iter().collect::<Vec<_>>(),
                "workflows": group.workflows.iter().collect::<Vec<_>>(),
                "visibilities": group.visibilities.iter().collect::<Vec<_>>(),
                "entries": group.entries.iter().map(|entry| serde_json::json!({
                    "log_index": entry.log_index,
                    "kind": entry.kind,
                    "api_version": entry.api_version,
                })).collect::<Vec<_>>(),
            })
        })
        .collect();

    let value = serde_json::json!({ "groups": groups });
    println!(
        "{}",
        serde_json::to_string_pretty(&value).expect("serialize findings")
    );
}

impl LeakGroup {
    fn is_private(&self) -> bool {
        self.visibilities
            .iter()
            .any(|visibility| !visibility.eq_ignore_ascii_case("public"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::client::identity::RepoOwner;

    fn org(slug: &str) -> IdentityKey {
        IdentityKey::Org(slug.to_owned())
    }

    #[test]
    fn identity_key_prefers_org() {
        let with_org = FulcioIdentity {
            organization: Some(RepoOwner::slug("sigstore")),
            email: Some("ci@sigstore.dev".to_owned()),
            ..Default::default()
        };
        assert_eq!(IdentityKey::of(&with_org), Some(org("sigstore")));

        let email_only = FulcioIdentity {
            email: Some("dev@example.com".to_owned()),
            ..Default::default()
        };
        assert_eq!(
            IdentityKey::of(&email_only),
            Some(IdentityKey::Email("dev@example.com".to_owned()))
        );

        let neither = FulcioIdentity::default();
        assert_eq!(IdentityKey::of(&neither), None);
    }

    #[test]
    fn insert_groups_by_identity_key() {
        let mut store = LeakStore::default();

        let leak = LeakResult {
            log_index: 1,
            api_version: "0.0.2".into(),
            kind: "intoto".into(),
            identities: vec![FulcioIdentity {
                organization: Some(RepoOwner::slug("sigstore")),
                source_repo_uri: Some("https://github.com/sigstore/cosign".into()),
                workflow_name: Some("release.yml".into()),
                source_repo_visibility: Some("public".into()),
                ..Default::default()
            }],
        };
        store.insert(leak);

        assert!(store.groups.contains_key(&org("sigstore")));
        assert_eq!(store.groups[&org("sigstore")].entries.len(), 1);
        assert!(
            store.groups[&org("sigstore")]
                .repos
                .contains("https://github.com/sigstore/cosign")
        );
        assert!(
            store.groups[&org("sigstore")]
                .workflows
                .contains("release.yml")
        );
        assert!(!store.groups[&org("sigstore")].is_private());

        // No org → keyed by email
        let leak_no_org = LeakResult {
            log_index: 2,
            api_version: "0.0.2".into(),
            kind: "intoto".into(),
            identities: vec![FulcioIdentity {
                email: Some("user@test.com".into()),
                ..Default::default()
            }],
        };
        store.insert(leak_no_org);
        assert!(
            store
                .groups
                .contains_key(&IdentityKey::Email("user@test.com".into()))
        );
    }

    #[test]
    fn test_org_wins_over_email() {
        let mut store = LeakStore::default();
        let leak = LeakResult {
            log_index: 1,
            api_version: "0.0.2".into(),
            kind: "hashedrekord".into(),
            identities: vec![FulcioIdentity {
                organization: Some(RepoOwner::slug("sigstore")),
                email: Some("ci-bot@sigstore.dev".into()),
                ..Default::default()
            }],
        };
        store.insert(leak);

        assert!(store.groups.contains_key(&org("sigstore")), "org key wins");
        assert!(
            !store
                .groups
                .contains_key(&IdentityKey::Email("ci-bot@sigstore.dev".into()))
        );
        assert_eq!(store.groups[&org("sigstore")].entries.len(), 1);
    }

    #[test]
    fn test_same_identity_collapses_indices() {
        let mut store = LeakStore::default();
        for idx in [10u64, 20, 30] {
            let leak = LeakResult {
                log_index: idx,
                api_version: "0.0.2".into(),
                kind: "hashedrekord".into(),
                identities: vec![FulcioIdentity {
                    email: Some("ci@example.com".into()),
                    ..Default::default()
                }],
            };
            store.insert(leak);
        }

        let key = IdentityKey::Email("ci@example.com".into());
        let group = &store.groups[&key];
        assert_eq!(group.entries.len(), 3);
        let indices: Vec<u64> = group.entries.iter().map(|e| e.log_index).collect();
        assert_eq!(indices, vec![10, 20, 30]);
    }

    #[test]
    fn test_private_only_group() {
        let mut store = LeakStore::default();
        store.insert(LeakResult {
            log_index: 1,
            api_version: "0.0.2".into(),
            kind: "hashedrekord".into(),
            identities: vec![FulcioIdentity {
                organization: Some(RepoOwner::slug("internal-org")),
                source_repo_visibility: Some("private".into()),
                ..Default::default()
            }],
        });

        assert!(store.groups[&org("internal-org")].is_private());
    }
}
