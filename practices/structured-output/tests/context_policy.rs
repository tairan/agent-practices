use structured_output::{
    AccessDecision, ContextBuildError, ContextBuilder, ContextItem, ContextRole, TrustLevel,
};

fn item(role: ContextRole, content: &str, trust: TrustLevel) -> ContextItem {
    item_with(
        role,
        content,
        trust,
        AccessDecision::allowed("acl-allow-test"),
        100,
    )
}

fn item_with(
    role: ContextRole,
    content: &str,
    trust: TrustLevel,
    access: AccessDecision,
    observed_at: u64,
) -> ContextItem {
    ContextItem::new(
        role,
        content,
        "fixture://context-policy",
        "case:context-policy",
        trust,
        "tenant-a",
        access,
        observed_at,
        "v1",
        None,
        "required test input",
    )
}

#[test]
fn rejects_trust_escalation_denied_future_and_empty_items() {
    let escalated = item(ContextRole::System, "x", TrustLevel::UntrustedData);
    assert!(matches!(
        ContextBuilder::new("tenant-a", 150, 100)
            .add(escalated)
            .build(),
        Err(ContextBuildError::TrustRoleMismatch)
    ));

    let denied = item_with(
        ContextRole::User,
        "x",
        TrustLevel::UntrustedData,
        AccessDecision::denied("acl-test-deny"),
        100,
    );
    assert!(matches!(
        ContextBuilder::new("tenant-a", 150, 100)
            .add(denied)
            .build(),
        Err(ContextBuildError::AccessDenied)
    ));

    let future = item_with(
        ContextRole::User,
        "x",
        TrustLevel::UntrustedData,
        AccessDecision::allowed("acl-allow-test"),
        151,
    );
    assert!(matches!(
        ContextBuilder::new("tenant-a", 150, 100)
            .add(future)
            .build(),
        Err(ContextBuildError::FutureDated)
    ));

    let empty = item(ContextRole::User, "   ", TrustLevel::UntrustedData);
    assert!(matches!(
        ContextBuilder::new("tenant-a", 150, 100).add(empty).build(),
        Err(ContextBuildError::LowValue)
    ));
}
