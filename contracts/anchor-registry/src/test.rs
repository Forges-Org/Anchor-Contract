use super::{
    AnchorMetadata, AnchorRegistry, AnchorRegistryClient, HealthAttestation, RegistryError,
};
use soroban_sdk::{testutils::Address as _, Address, Env, String, Symbol};

fn setup() -> (Env, AnchorRegistryClient<'static>, Address, Address, Symbol) {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let reporter = Address::generate(&env);
    let anchor_id = Symbol::new(&env, "anchor_one");
    let contract_id = env.register(AnchorRegistry, ());
    let client = AnchorRegistryClient::new(&env, &contract_id);
    client.initialize(&admin);
    (env, client, admin, reporter, anchor_id)
}

fn metadata(env: &Env, reporter: &Address) -> AnchorMetadata {
    AnchorMetadata {
        name: String::from_str(env, "Anchor One"),
        home_domain: String::from_str(env, "anchor.example"),
        asset_code: Symbol::new(env, "USD"),
        country_code: Symbol::new(env, "USA"),
        reporter: reporter.clone(),
    }
}

fn register_anchor(
    env: &Env,
    client: &AnchorRegistryClient,
    admin: &Address,
    reporter: &Address,
    anchor_id: &Symbol,
) {
    client.register_anchor(admin, anchor_id, &metadata(env, reporter));
}

#[test]
fn registry_stores_metadata_and_reports_latest_health() {
    let (env, client, admin, reporter, anchor_id) = setup();
    register_anchor(&env, &client, &admin, &reporter, &anchor_id);
    env.ledger().with_mut(|ledger| ledger.timestamp = 100);

    let first = client.attest_health(&reporter, &anchor_id, &82, &90);
    env.ledger().with_mut(|ledger| ledger.timestamp = 110);
    let second = client.attest_health(&reporter, &anchor_id, &95, &105);

    assert_eq!(client.get_anchor(&anchor_id), metadata(&env, &reporter));
    assert_eq!(client.get_health(&anchor_id), second);
    assert_eq!(first.sequence, 0);
    assert_eq!(client.get_attestation_count(&anchor_id), 2);
}

#[test]
fn history_uses_zero_based_bounded_pages() {
    let (env, client, admin, reporter, anchor_id) = setup();
    register_anchor(&env, &client, &admin, &reporter, &anchor_id);
    env.ledger().with_mut(|ledger| ledger.timestamp = 30);
    for (timestamp, score) in [(10, 70), (20, 80), (30, 90)] {
        client.attest_health(&reporter, &anchor_id, &score, &timestamp);
    }

    let page = client.health_history(&anchor_id, &1, &2);
    assert_eq!(page.len(), 2);
    assert_eq!(page.get(0).unwrap().sequence, 1);
    assert_eq!(page.get(1).unwrap().score, 90);
}

#[test]
fn invalid_score_and_future_timestamp_are_rejected() {
    let (env, client, admin, reporter, anchor_id) = setup();
    register_anchor(&env, &client, &admin, &reporter, &anchor_id);
    env.ledger().with_mut(|ledger| ledger.timestamp = 10);

    assert_eq!(
        client.try_attest_health(&reporter, &anchor_id, &101, &10),
        Err(Ok(RegistryError::InvalidScore))
    );
    assert_eq!(
        client.try_attest_health(&reporter, &anchor_id, &90, &11),
        Err(Ok(RegistryError::InvalidTimestamp))
    );
}

#[test]
fn non_increasing_timestamps_and_invalid_pages_are_rejected() {
    let (env, client, admin, reporter, anchor_id) = setup();
    register_anchor(&env, &client, &admin, &reporter, &anchor_id);
    env.ledger().with_mut(|ledger| ledger.timestamp = 10);
    client.attest_health(&reporter, &anchor_id, &80, &10);

    assert_eq!(
        client.try_attest_health(&reporter, &anchor_id, &81, &10),
        Err(Ok(RegistryError::InvalidTimestamp))
    );
    assert_eq!(
        client.try_health_history(&anchor_id, &0, &0),
        Err(Ok(RegistryError::InvalidHistoryPage))
    );
    assert_eq!(
        client.try_health_history(&anchor_id, &2, &1),
        Err(Ok(RegistryError::InvalidHistoryPage))
    );
}

#[test]
fn unknown_anchor_cannot_be_attested_or_queried() {
    let (_env, client, _admin, reporter, anchor_id) = setup();
    assert_eq!(
        client.try_attest_health(&reporter, &anchor_id, &80, &1),
        Err(Ok(RegistryError::UnknownAnchor))
    );
    assert_eq!(
        client.try_get_attestation_count(&anchor_id),
        Err(Ok(RegistryError::UnknownAnchor))
    );
}