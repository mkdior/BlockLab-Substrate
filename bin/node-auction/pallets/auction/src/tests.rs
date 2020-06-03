// Creating mock runtime here

use crate::*;
use frame_support::{
    assert_err, assert_ok, impl_outer_event, impl_outer_origin, parameter_types,
    traits::{BalanceStatus, OnInitialize, OnFinalize},
};
use frame_system::{self as system, RawOrigin};
use orml_traits::auction::*;
use pallet_balances::{self as balances};
use sp_core::{LogLevel, H256};
use sp_io::{self as io, logging::log};
use sp_runtime::{
    testing::Header,
    traits::{BlakeTwo256, IdentityLookup},
    Perbill,
};

impl_outer_origin! {
    pub enum Origin for AuctionTestRuntime {}
}

// For testing the pallet, we construct most of a mock runtime. This means
// first constructing a configuration type (`Test`) which `impl`s each of the
// configuration traits of pallets we want to use.
#[derive(Clone, Eq, PartialEq, Debug)]
pub struct AuctionTestRuntime;

parameter_types! {
    pub const BlockHashCount: u64 = 250;
    pub const MaximumBlockWeight: u32 = 1024;
    pub const MaximumBlockLength: u32 = 2 * 1024;
    pub const AvailableBlockRatio: Perbill = Perbill::from_percent(75);

    pub const ExistentialDeposit: u64 = 1;
    pub const TransferFee: u64 = 0;
    pub const CreationFee: u64 = 0;
}

pub type AccountId = u64;
pub type Balance = u64;
pub type BlockNumber = u64;
pub type AuctionId = u64;

impl system::Trait for AuctionTestRuntime {
    type Origin = Origin;
    type Call = ();
    type Index = u64;
    type BlockNumber = u64;
    type Hash = H256;
    type Hashing = BlakeTwo256;
    type AccountId = u64;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Header = Header;
    type Event = (); //AuctionTestEvent;
    type BlockHashCount = BlockHashCount;
    type MaximumBlockWeight = MaximumBlockWeight;
    type MaximumBlockLength = MaximumBlockLength;
    type AvailableBlockRatio = AvailableBlockRatio;
    type Version = ();
    type ModuleToIndex = ();
    type AccountData = balances::AccountData<u64>;
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type DbWeight = ();
}

pub struct Handler;

impl AuctionHandler<AccountId, Balance, BlockNumber, AuctionId> for Handler {
    fn on_new_bid(
        _now: BlockNumber,
        _id: AuctionId,
        _new_bid: (AccountId, Balance),
        _last_bid: Option<(AccountId, Balance)>,
    ) -> OnNewBidResult<BlockNumber> {
        // Test the sending of balances here.
        OnNewBidResult {
            accept_bid: true,
            auction_end: None,
        }
    }

    fn on_auction_ended(_id: AuctionId, _winner: Option<(AccountId, Balance)>) {}
}

impl balances::Trait for AuctionTestRuntime {
    type Balance = Balance;
    type Event = (); //AuctionTestEvent;
    type DustRemoval = ();
    type ExistentialDeposit = ExistentialDeposit;
    type AccountStore = system::Module<AuctionTestRuntime>;
}

impl Trait for AuctionTestRuntime {
    type Event = (); //AuctionTestEvent;
    type Currency = balances::Module<Self>;
    type AuctionId = AccountId;
    type Handler = Handler;
}

pub type System = system::Module<AuctionTestRuntime>;
pub type Balances = balances::Module<AuctionTestRuntime>;
pub type AuctionModule = Module<AuctionTestRuntime>;

//mod auction_events {
//    pub use crate::Event;
//}
//
//impl_outer_event! {
//    pub enum AuctionTestEvent for AuctionTestRuntime {
//        auction_events<T>,
//        system<T>,
//        balances<T>,
//    }
//}

// Simulating block production for the general auction tests
fn run_to_block(n: u64) {
    while System::block_number() < n {
        AuctionModule::on_finalize(System::block_number());
        System::on_finalize(System::block_number());
        System::set_block_number(System::block_number() + 1);
        System::on_initialize(System::block_number());

        // If needed, initialize our auction
        // AuctionModule::on_intitialize(System::block_number());
    }
}

pub struct EnvBuilder {
    balances: Vec<(u64, u64)>,
    auctions: Vec<(AccountId, BlockNumber, BlockNumber)>,
}

impl EnvBuilder {
    pub fn new() -> Self {
        Self {
            balances: vec![(1, 20000), (2, 20000), (3, 20000), (4, 20000), (5, 20000)],
            auctions: vec![(1, 1, 50), (2, 1, 50), (3, 1, 50), (4, 1, 50)],
        }
    }
}

// This function basically just builds a genesis storage key/value store according to
// our desired mockup.
pub fn new_test_ext() -> sp_io::TestExternalities {
    let core = EnvBuilder::new();
    let mut t = system::GenesisConfig::default()
        .build_storage::<AuctionTestRuntime>()
        .unwrap();
    balances::GenesisConfig::<AuctionTestRuntime> {
        balances: core.balances,
    }
    .assimilate_storage(&mut t)
    .unwrap();
    GenesisConfig::<AuctionTestRuntime> {
        _auctions: core.auctions,
    }
    .assimilate_storage(&mut t)
    .unwrap();
    t.into()
}

#[test]
fn new_test_ext_behaves() {
    new_test_ext().execute_with(|| {
        assert_eq!(Balances::free_balance(&1), 20000);
    })
}

// Currency
/////////////////////////////////////////////////////////
#[test]
fn new_test_ext_transfer() {
    new_test_ext().execute_with(|| {
        assert_ok!(Balances::transfer(Origin::signed(1), 2, 1000));
        assert_eq!(Balances::free_balance(&2), 21000);
        assert_ok!(Balances::transfer(Origin::signed(2), 1, 1000));
        assert_eq!(Balances::free_balance(&1), 20000);
    })
}
/////////////////////////////////////////////////////////
// Reservable
/////////////////////////////////////////////////////////
#[test]
fn new_test_ext_can_reserve() {
    new_test_ext().execute_with(|| {
        assert_eq!(Balances::can_reserve(&1, 29), true);
        assert_eq!(Balances::can_reserve(&1, 20001), false);
    })
}

#[test]
fn new_test_ext_reserve() {
    new_test_ext().execute_with(|| {
        assert_ok!(Balances::reserve(&1, 10000));
        assert_eq!(Balances::free_balance(&1), 10000);
        assert!(Balances::reserve(&1, 31000).is_err());
    })
}
//https://substrate.dev/rustdocs/master/src/frame_support/traits.rs.html#725-783
#[test]
fn new_test_ext_unreserver() {
    new_test_ext().execute_with(|| {
        assert_ok!(Balances::reserve(&1, 10000));
        assert_eq!(Balances::free_balance(&1), 10000);
        assert_eq!(Balances::unreserve(&1, 10000), 0);
        assert_ok!(Balances::reserve(&1, 20000));
    })
}

#[test]
fn new_test_ext_slash_reserve() {
    new_test_ext().execute_with(|| {
        assert_ok!(Balances::reserve(&1, 10000));
        assert_eq!(Balances::free_balance(&1), 10000);
        assert_eq!(Balances::reserved_balance(&1), 10000);
        let slash_res = Balances::slash_reserved(&1, 10000);
        assert_eq!(Balances::reserved_balance(&1), 0);
        assert_eq!(Balances::free_balance(&1), 10000);
    })
}

#[test]
fn new_test_ext_repatriate_reserved() {
    new_test_ext().execute_with(|| {
        assert_ok!(Balances::reserve(&1, 10000));
        assert_eq!(Balances::free_balance(&1), 10000);
        assert_eq!(Balances::reserved_balance(&1), 10000);
        assert_ok!(Balances::repatriate_reserved(
            &1,
            &2,
            Balances::reserved_balance(&1),
            BalanceStatus::Free
        ));
        assert_eq!(Balances::reserved_balance(&1), 0);
        assert_eq!(Balances::free_balance(&1), 10000);
        assert_eq!(Balances::free_balance(&2), 30000);
    })
}
///////////////////////////////////////////////////////
// Auction related tests
///////////////////////////////////////////////////////
#[test]
fn new_test_ext_new_auction() {
    new_test_ext().execute_with(|| {
        assert!(AuctionModule::auction_exists(1), true);
        assert!(AuctionModule::auction_exists(2), true);
        assert!(AuctionModule::auction_exists(3), true);
        assert!(AuctionModule::auction_exists(0), true);
    })
}

#[test]
fn new_test_ext_auction_info() {
    new_test_ext().execute_with(|| {
        // Auction Active up untill block n 50
        assert!(AuctionModule::auction_exists(1), true);
        let info = AuctionModule::auction_info(1);
        println!("{:?}", info); 
        run_to_block(90);
        println!("{}", System::block_number());
        assert!(AuctionModule::auction_exists(1), true);
        
    })
}
