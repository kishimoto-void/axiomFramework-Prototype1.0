//! Projectors — TokenOnly + Minimal (PLP-R hierarchy).

use crate::hash_ser::{build_canonical_payload, dual_hash};
use crate::{
    Annotation, AnnotationKind, CanonicalState, PlpError, Projection, ProjectionHeader,
    PAYLOAD_VERSION, PROTOCOL,
};
use axiom_pss::NormalizedInput;
use std::collections::BTreeMap;

#[derive(Debug, Clone, Default)]
pub struct ProjectOptions {
    pub capsule_id: String,
    pub parent_id: Option<String>,
    pub clock: u64,
    pub sequence: u64,
    pub timestamp_ns: u64,
    pub source: String,
}

impl ProjectOptions {
    pub fn with_id(id: impl Into<String>) -> Self {
        Self {
            capsule_id: id.into(),
            source: "plp-r".into(),
            ..Default::default()
        }
    }
}

fn split_tokens(text: &str, language: &str) -> Vec<String> {
    let seps = " \t\n\r。、．，,!?！？";
    let mut parts = Vec::new();
    let mut buf = String::new();
    for ch in text.chars() {
        if seps.contains(ch) || ch.is_whitespace() {
            if !buf.is_empty() {
                parts.push(std::mem::take(&mut buf));
            }
        } else {
            buf.push(ch);
        }
    }
    if !buf.is_empty() {
        parts.push(buf);
    }
    if language == "en" {
        parts.into_iter().map(|s| s.to_lowercase()).collect()
    } else {
        parts
    }
}

fn language_of(input: &NormalizedInput) -> String {
    input.language_hint.clone().unwrap_or_else(|| "en".into())
}

fn header_from(opts: &ProjectOptions) -> ProjectionHeader {
    ProjectionHeader {
        protocol: PROTOCOL.into(),
        version: PAYLOAD_VERSION.into(),
        capsule_id: opts.capsule_id.clone(),
        parent_id: opts.parent_id.clone(),
        clock: opts.clock,
        sequence: opts.sequence,
        timestamp_ns: opts.timestamp_ns,
        source: if opts.source.is_empty() {
            "plp-r".into()
        } else {
            opts.source.clone()
        },
        hash_algorithm: "sha256".into(),
    }
}

fn seal(raw_text: String, header: ProjectionHeader, canonical: CanonicalState) -> Projection {
    let payload = build_canonical_payload(&header, &canonical);
    let (raw_hash, canonical_hash) = dual_hash(&raw_text, &payload);
    Projection {
        header,
        raw_text,
        canonical,
        raw_hash,
        canonical_hash,
    }
}

/// TokenOnlyProjector — baseline. annotations = [].
pub fn project_token_only(
    input: &NormalizedInput,
    opts: ProjectOptions,
) -> Result<Projection, PlpError> {
    if input.text.trim().is_empty() {
        return Err(PlpError::EmptyInput);
    }
    let language = language_of(input);
    let tokens = split_tokens(&input.text, &language);
    let mut meta = BTreeMap::new();
    meta.insert("annotation_status".into(), "none".into());
    meta.insert("projector".into(), "TokenOnly".into());
    let canonical = CanonicalState {
        version: PAYLOAD_VERSION.into(),
        language,
        tokens,
        annotations: vec![],
        meta,
    };
    Ok(seal(input.text.clone(), header_from(&opts), canonical))
}

/// MinimalProjector — demo heuristics. Candidates only.
pub fn project_minimal(
    input: &NormalizedInput,
    opts: ProjectOptions,
) -> Result<Projection, PlpError> {
    if input.text.trim().is_empty() {
        return Err(PlpError::EmptyInput);
    }
    let language = language_of(input);
    let tokens = split_tokens(&input.text, &language);
    let annotations = annotate_minimal(&tokens, &language);
    let mut meta = BTreeMap::new();
    meta.insert(
        "annotation_status".into(),
        "canonical_projection_candidate".into(),
    );
    meta.insert("projector".into(), "Minimal".into());
    let canonical = CanonicalState {
        version: PAYLOAD_VERSION.into(),
        language,
        tokens,
        annotations,
        meta,
    };
    Ok(seal(input.text.clone(), header_from(&opts), canonical))
}

fn annotate_minimal(tokens: &[String], language: &str) -> Vec<Annotation> {
    let actions_en = [
        "add", "enable", "publish", "set", "run", "create", "update", "delete",
    ];
    let actions_ja = ["追加", "実行", "設定", "作成", "更新", "削除", "有効化"];
    let loc_en = ["in", "on", "at", "from", "to"];
    let loc_ja = ["で", "に", "へ", "から"];

    let mut anns = Vec::new();
    let mut entity_once = false;

    for (i, t) in tokens.iter().enumerate() {
        let is_action = if language == "ja" {
            actions_ja.contains(&t.as_str())
        } else {
            actions_en.contains(&t.as_str())
        };
        if is_action {
            anns.push(Annotation::new(AnnotationKind::Action, t.clone()));
            continue;
        }

        if language == "ja" && t.chars().count() >= 2 && !entity_once {
            anns.push(Annotation::new(AnnotationKind::Entity, t.clone()));
            entity_once = true;
        }

        let is_loc = if language == "ja" {
            loc_ja.contains(&t.as_str())
        } else {
            loc_en.contains(&t.as_str())
        };
        if is_loc {
            if let Some(next) = tokens.get(i + 1) {
                anns.push(Annotation::with_key(
                    AnnotationKind::Location,
                    next.clone(),
                    t.clone(),
                ));
            }
        }
    }

    let mut seen = std::collections::BTreeSet::new();
    anns.retain(|a| {
        let k = (
            a.kind.clone(),
            a.value.clone(),
            a.key.clone().unwrap_or_default(),
        );
        seen.insert(k)
    });
    anns
}

pub fn project_text_token_only(
    raw: &str,
    opts: ProjectOptions,
) -> Result<Projection, Box<dyn std::error::Error>> {
    let n = axiom_pss::normalize(raw)?;
    Ok(project_token_only(&n, opts)?)
}

pub fn project_text_minimal(
    raw: &str,
    opts: ProjectOptions,
) -> Result<Projection, Box<dyn std::error::Error>> {
    let n = axiom_pss::normalize(raw)?;
    Ok(project_minimal(&n, opts)?)
}
