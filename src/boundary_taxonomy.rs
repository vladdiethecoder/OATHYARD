/// Boundary taxonomy labels for frontier-research artifacts.
///
/// Source policy: `docs/design/GAME_CANON.md` requires deterministic combat truth,
/// replay authority, and no nondeterministic presentation/AI writes into gameplay
/// truth. The `/goal` frontier-research layer narrows that into three artifact
/// labels:
/// - AI-assisted candidate content before every freeze condition is satisfied;
/// - authoritative post-freeze content after all five freeze conditions pass;
/// - deterministic artifacts never touched by AI.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ArtifactOrigin {
    AiAssisted,
    DeterministicNeverAi,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BoundaryFreezeState {
    pub frozen: bool,
    pub deterministic: bool,
    pub hashed: bool,
    pub replayable: bool,
    pub cross_platform_verified: bool,
}

impl BoundaryFreezeState {
    pub const fn all_conditions_passed(self) -> bool {
        self.frozen
            && self.deterministic
            && self.hashed
            && self.replayable
            && self.cross_platform_verified
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ArtifactBoundaryMetadata {
    pub origin: ArtifactOrigin,
    pub freeze: BoundaryFreezeState,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BoundaryTaxonomyLabel {
    AiAssistedPreFreeze,
    AuthoritativePostFreeze,
    PurelyDeterministicNeverAi,
}

impl BoundaryTaxonomyLabel {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::AiAssistedPreFreeze => "AI-assisted-pre-freeze",
            Self::AuthoritativePostFreeze => "authoritative-post-freeze",
            Self::PurelyDeterministicNeverAi => "purely-deterministic-never-AI",
        }
    }
}

/// Pure boundary-taxonomy classifier for artifact metadata.
///
/// AI-assisted artifacts become authoritative only when frozen, deterministic,
/// hashed, replayable, and cross-platform verified are all true. Deterministic
/// artifacts that were never touched by AI stay in the deterministic lane even
/// when they have not gone through an AI-content freeze pipeline.
pub const fn tag_artifact_boundary(metadata: &ArtifactBoundaryMetadata) -> BoundaryTaxonomyLabel {
    match metadata.origin {
        ArtifactOrigin::DeterministicNeverAi => BoundaryTaxonomyLabel::PurelyDeterministicNeverAi,
        ArtifactOrigin::AiAssisted if metadata.freeze.all_conditions_passed() => {
            BoundaryTaxonomyLabel::AuthoritativePostFreeze
        }
        ArtifactOrigin::AiAssisted => BoundaryTaxonomyLabel::AiAssistedPreFreeze,
    }
}
