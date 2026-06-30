use oathyard::{
    tag_artifact_boundary, ArtifactBoundaryMetadata, ArtifactOrigin, BoundaryFreezeState,
    BoundaryTaxonomyLabel,
};

fn all_freeze_conditions_passed() -> BoundaryFreezeState {
    BoundaryFreezeState {
        frozen: true,
        deterministic: true,
        hashed: true,
        replayable: true,
        cross_platform_verified: true,
    }
}

fn no_freeze_conditions_passed() -> BoundaryFreezeState {
    BoundaryFreezeState {
        frozen: false,
        deterministic: false,
        hashed: false,
        replayable: false,
        cross_platform_verified: false,
    }
}

#[test]
fn ai_assisted_unfrozen_artifact_is_pre_freeze() {
    let metadata = ArtifactBoundaryMetadata {
        origin: ArtifactOrigin::AiAssisted,
        freeze: no_freeze_conditions_passed(),
    };

    let label = tag_artifact_boundary(&metadata);

    assert_eq!(label, BoundaryTaxonomyLabel::AiAssistedPreFreeze);
    assert_eq!(label.as_str(), "AI-assisted-pre-freeze");
}

#[test]
fn ai_assisted_artifact_with_all_five_freeze_conditions_is_authoritative_post_freeze() {
    let metadata = ArtifactBoundaryMetadata {
        origin: ArtifactOrigin::AiAssisted,
        freeze: all_freeze_conditions_passed(),
    };

    let label = tag_artifact_boundary(&metadata);

    assert_eq!(label, BoundaryTaxonomyLabel::AuthoritativePostFreeze);
    assert_eq!(label.as_str(), "authoritative-post-freeze");
}

#[test]
fn deterministic_never_ai_artifact_gets_pure_deterministic_label_even_when_unfrozen() {
    let metadata = ArtifactBoundaryMetadata {
        origin: ArtifactOrigin::DeterministicNeverAi,
        freeze: no_freeze_conditions_passed(),
    };

    let label = tag_artifact_boundary(&metadata);

    assert_eq!(label, BoundaryTaxonomyLabel::PurelyDeterministicNeverAi);
    assert_eq!(label.as_str(), "purely-deterministic-never-AI");
}

#[test]
fn ai_assisted_artifact_with_partial_freeze_remains_pre_freeze() {
    let metadata = ArtifactBoundaryMetadata {
        origin: ArtifactOrigin::AiAssisted,
        freeze: BoundaryFreezeState {
            frozen: true,
            deterministic: true,
            hashed: true,
            replayable: false,
            cross_platform_verified: false,
        },
    };

    let label = tag_artifact_boundary(&metadata);

    assert_eq!(label, BoundaryTaxonomyLabel::AiAssistedPreFreeze);
    assert_eq!(label.as_str(), "AI-assisted-pre-freeze");
}

#[test]
fn ai_assisted_artifact_missing_any_single_freeze_condition_remains_pre_freeze() {
    let mut cases = [all_freeze_conditions_passed(); 5];
    cases[0].frozen = false;
    cases[1].deterministic = false;
    cases[2].hashed = false;
    cases[3].replayable = false;
    cases[4].cross_platform_verified = false;

    for freeze in cases {
        let metadata = ArtifactBoundaryMetadata {
            origin: ArtifactOrigin::AiAssisted,
            freeze,
        };

        assert_eq!(
            tag_artifact_boundary(&metadata),
            BoundaryTaxonomyLabel::AiAssistedPreFreeze,
            "AI-assisted artifacts are authoritative only after all five freeze conditions pass"
        );
    }
}
