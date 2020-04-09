// Creating mock runtime here

use crate::*;
use frame_support::{assert_ok, assert_err, impl_outer_origin, impl_outer_event, parameter_types};
use frame_system::{self as system, RawOrigin};
use sp_core::H256;
use sp_io;
use sp_runtime::{
    testing::Header,
    traits::{BlakeTwo256, IdentityLookup},
    Perbill,
};
use pallet_balances::{self as balances};

impl_outer_origin! {
    pub enum Origin for AuctionTestRuntime {}
}

// For testing the pallet, we construct most of a mock runtime. This means
// first constructing a configuration type (`Test`) which `impl`s each of the
// configuration traits of pallets we want to use.
#[derive(Clone, Eq, PartialEq)]
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
    type Event = AuctionTestEvent;
    type BlockHashCount = BlockHashCount;
    type MaximumBlockWeight = MaximumBlockWeight;
    type MaximumBlockLength = MaximumBlockLength;
    type AvailableBlockRatio = AvailableBlockRatio;
    type Version = ();
    type ModuleToIndex = ();
    type AccountData = balances::AccountData<u64>;
    type OnNewAccount = ();
    type OnKilledAccount = ();
}

impl balances::Trait for AuctionTestRuntime {
    type Balance = u64;
    type Event = AuctionTestEvent;
    type DustRemoval = ();
    type ExistentialDeposit = ExistentialDeposit;
    type AccountStore = system::Module<AuctionTestRuntime>;
}

mod charity {
    pub use crate::Event;
}

impl_outer_event! {
    pub enum AuctionTestEvent for AuctionTestRuntime {
        system<T>,
        charity<T>,
        balances<T>,
    }
}

impl Trait for AuctionTestRuntime {
    type Event = AuctionTestEvent;
    type Currency = balances::Module<Self>;
}

pub type System = system::Module<AuctionTestRuntime>;
pub type Balances = balances::Module<AuctionTestRuntime>;
pub type AuctionModule = Module<AuctionTestRuntime>;

// This function basically just builds a genesis storage key/value store according to
// our desired mockup.
pub fn new_test_ext() -> sp_io::TestExternalities {
    let mut t = system::GenesisConfig::default()
        .build_storage::<AuctionTestRuntime>()
        .unwrap();
    balances::GenesisConfig::<AuctionTestRuntime> {
        balances: vec![(1, 13), (2, 11), (3, 1), (4, 3), (5, 19)],
    }
    .assimilate_storage(&mut t)
    .unwrap();

    t.into()
}



#[test]
fn new_test_ext_behaves() {
    new_test_ext().execute_with(|| {
        assert_eq!(Balances::free_balance(&1), 13);
    })
}
