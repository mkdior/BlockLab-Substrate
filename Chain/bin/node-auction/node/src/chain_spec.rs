use node_auction_runtime::{
    AccountId, AuctionModuleConfig, AuraConfig, BalancesConfig, BlockNumber,
    GeneralInformationContainer, GenesisConfig, GrandpaConfig, Signature, SudoConfig, SystemConfig,
    WASM_BINARY,
};
use sc_service::ChainType;
use sp_consensus_aura::sr25519::AuthorityId as AuraId;
use sp_core::{sr25519, Pair, Public};
use sp_finality_grandpa::AuthorityId as GrandpaId;
use sp_runtime::traits::{IdentifyAccount, Verify};

// Note this is the URL for the telemetry server
//const STAGING_TELEMETRY_URL: &str = "wss://telemetry.polkadot.io/submit/";

/// Specialized `ChainSpec`. This is a specialization of the general Substrate ChainSpec type.
pub type ChainSpec = sc_service::GenericChainSpec<GenesisConfig>;

/// Helper function to generate a crypto pair from seed
pub fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
    TPublic::Pair::from_string(&format!("//{}", seed), None)
        .expect("static values are valid; qed")
        .public()
}

type AccountPublic = <Signature as Verify>::Signer;

/// Helper function to generate an account ID from seed
pub fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId
where
    AccountPublic: From<<TPublic::Pair as Pair>::Public>,
{
    AccountPublic::from(get_from_seed::<TPublic>(seed)).into_account()
}

/// Helper function to generate an authority key for Aura
pub fn authority_keys_from_seed(s: &str) -> (AuraId, GrandpaId) {
    (get_from_seed::<AuraId>(s), get_from_seed::<GrandpaId>(s))
}

/// Our Development testnet which allows for a single validator to produce blocks. In this case we
/// have Alice as the intial authority and as the root key.
pub fn development_config() -> ChainSpec {
    ChainSpec::from_genesis(
        "Development",
        "dev",
        ChainType::Development,
        || {
            testnet_genesis(
                vec![authority_keys_from_seed("Alice")],
                get_account_id_from_seed::<sr25519::Public>("Alice"),
                vec![
                    // Begin Terminals //
                    get_account_id_from_seed::<sr25519::Public>("Alice"),
                    get_account_id_from_seed::<sr25519::Public>("Bob"),
                    // End Terminals -- Rest are barges //
                    get_account_id_from_seed::<sr25519::Public>("Charlie"),
                    get_account_id_from_seed::<sr25519::Public>("Dave"),
                    get_account_id_from_seed::<sr25519::Public>("Eve"),
                    get_account_id_from_seed::<sr25519::Public>("Ferdie"),
                    get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
                    get_account_id_from_seed::<sr25519::Public>("Charlie//stash"),
                    get_account_id_from_seed::<sr25519::Public>("Dave//stash"),
                    get_account_id_from_seed::<sr25519::Public>("Eve//stash"),
                    get_account_id_from_seed::<sr25519::Public>("Ferdie//stash"),
                    get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
                ],
                vec![
                    (
                        get_account_id_from_seed::<sr25519::Public>("Alice"),   // Terminal
                        get_account_id_from_seed::<sr25519::Public>("Charlie"), // Barge
                    ),
                    (
                        get_account_id_from_seed::<sr25519::Public>("Bob"),
                        get_account_id_from_seed::<sr25519::Public>("Charlie"),
                    ),
                    (
                        get_account_id_from_seed::<sr25519::Public>("Alice"),
                        get_account_id_from_seed::<sr25519::Public>("Dave"),
                    ),
                    (
                        get_account_id_from_seed::<sr25519::Public>("Bob"),
                        get_account_id_from_seed::<sr25519::Public>("Dave"),
                    ),
                    (
                        get_account_id_from_seed::<sr25519::Public>("Alice"),
                        get_account_id_from_seed::<sr25519::Public>("Eve"),
                    ),
                    (
                        get_account_id_from_seed::<sr25519::Public>("Bob"),
                        get_account_id_from_seed::<sr25519::Public>("Eve"),
                    ),
                    (
                        get_account_id_from_seed::<sr25519::Public>("Alice"),
                        get_account_id_from_seed::<sr25519::Public>("Ferdie"),
                    ),
                    (
                        get_account_id_from_seed::<sr25519::Public>("Bob"),
                        get_account_id_from_seed::<sr25519::Public>("Ferdie"),
                    ),
                ],
                true,
            )
        },
        vec![],
        None,
        None,
        None,
        None,
    )
}

/// Practically the same setup as our Development network but in this setup we have two validators,
/// Alice and Bob. To run the local testnet specify it by running "cargo run -- --chain=local".
pub fn local_testnet_config() -> ChainSpec {
    ChainSpec::from_genesis(
        "Local Testnet",
        "local_testnet",
        ChainType::Local,
        || {
            testnet_genesis(
                vec![
                    authority_keys_from_seed("Alice"),
                    authority_keys_from_seed("Bob"),
                ],
                get_account_id_from_seed::<sr25519::Public>("Alice"),
                vec![
                    get_account_id_from_seed::<sr25519::Public>("Alice"),
                    get_account_id_from_seed::<sr25519::Public>("Bob"),
                    get_account_id_from_seed::<sr25519::Public>("Charlie"),
                    get_account_id_from_seed::<sr25519::Public>("Dave"),
                    get_account_id_from_seed::<sr25519::Public>("Eve"),
                    get_account_id_from_seed::<sr25519::Public>("Ferdie"),
                    get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
                    get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
                    get_account_id_from_seed::<sr25519::Public>("Charlie//stash"),
                    get_account_id_from_seed::<sr25519::Public>("Dave//stash"),
                    get_account_id_from_seed::<sr25519::Public>("Eve//stash"),
                    get_account_id_from_seed::<sr25519::Public>("Ferdie//stash"),
                ],
                vec![(
                    get_account_id_from_seed::<sr25519::Public>("Alice"),
                    get_account_id_from_seed::<sr25519::Public>("Bob"),
                )],
                true,
            )
        },
        vec![],
        None,
        None,
        None,
        None,
    )
}

fn testnet_genesis(
    initial_authorities: Vec<(AuraId, GrandpaId)>,
    root_key: AccountId,
    endowed_accounts: Vec<AccountId>,
    auction_initiators: Vec<(AccountId, AccountId)>,
    _enable_println: bool,
) -> GenesisConfig {
    // Test auctions for our Development environment
    let mut test_auctions = vec![
        // Balance, Containers,  TEU, Start, End
        (500,   100,    100,    0,      50 ),
        (2040,  100,    100,    0,      100),
        (948,   50,     50,     10,     500),
        (2998,  500,    500,    100,    203),
        (293,   10,     10,     0,      70 ),
        (884,   200,    200,    20,     200),
        (503,   400,    400,    29,     388),
        (39894, 600,    600,    0,  1000000),
    ];

    GenesisConfig {
        system: Some(SystemConfig {
            code: WASM_BINARY.to_vec(),
            changes_trie_config: Default::default(),
        }),
        balances: Some(BalancesConfig {
            balances: endowed_accounts
                .iter()
                .cloned()
                .map(|k| (k, 1 << 60))
                .collect(),
        }),
        aura: Some(AuraConfig {
            authorities: initial_authorities.iter().map(|x| (x.0.clone())).collect(),
        }),
        grandpa: Some(GrandpaConfig {
            authorities: initial_authorities
                .iter()
                .map(|x| (x.1.clone(), 1))
                .collect(),
        }),
        sudo: Some(SudoConfig { key: root_key }),
        auction: Some(AuctionModuleConfig {
            _auctions: auction_initiators
                .iter()
                .cloned()
                .map(|x| {
                    // Make sure that the number of test_auctions == the number of test auction
                    // initiators container in auction_initiators. 
                    if let Some(current_auction) = test_auctions.pop() {
                        (
                            x.0,
                            x.1,
                            vec![current_auction.0, current_auction.1, current_auction.2],
                            current_auction.3,
                            current_auction.4,
                        )
                    } else {
                        (x.0, x.1, vec![2000, 200, 200], 0, 500)
                    }
                })
                .collect::<Vec<(
                    AccountId,
                    AccountId,
                    Vec<GeneralInformationContainer>,
                    BlockNumber,
                    BlockNumber,
                )>>(),
        }),
    }
}
