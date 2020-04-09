#![cfg_attr(not(feature = "std"), no_std)]

////////////////////////////////////////////////////////////////////////////////////////////////
// Imports
// /////////////////////////////////////////////////////////////////////////////////////////////
use frame_support::sp_runtime::{
    traits::{AtLeast32Bit, MaybeSerializeDeserialize, Member, One, Zero},
    DispatchResult,
};
use frame_support::{
    decl_error, decl_event, decl_module, decl_storage, dispatch,
    dispatch::Parameter,
    ensure,
    storage::IterableStorageDoubleMap,
    traits::{Currency, ExistenceRequirement::AllowDeath, ReservableCurrency},
};
use frame_system::{self as system, ensure_signed};
use orml_traits::auction::{Auction, AuctionHandler, AuctionInfo};
////////////////////////////////////////////////////////////////////////////////////////////////

type BalanceOf<T> = <<T as Trait>::Currency as Currency<<T as system::Trait>::AccountId>>::Balance;

/// The pallet's configuration trait.
pub trait Trait: system::Trait + Sized {
    /// The overarching event type.
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
    type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;
}

decl_storage! {trait Store for Module<T: Trait> as AuctionModule {}}

decl_event!(
    pub enum Event<T>
    where
        AccountId = <T as system::Trait>::AccountId,
        Balance = BalanceOf<T>,
        BlockNumber = <T as system::Trait>::BlockNumber,
    {
        LockFunds(AccountId, Balance, BlockNumber),
        UnlockFunds(AccountId, Balance, BlockNumber),
        TransferFunds(AccountId, AccountId, Balance, BlockNumber),
    }
);

decl_module! {
    /// The module declaration.
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        type Error = Error<T>;

        // Initializing events
        fn deposit_event() = default;

        pub fn lock_funds(origin, amount: BalanceOf<T>) -> DispatchResult {
            let target = ensure_signed(origin)?;

            //TODO(Hamza): Serve proper errors.
            T::Currency::reserve(&target, amount).map_err(|_| "Not able to reserve");

            let now = <system::Module<T>>::block_number();

            Self::deposit_event(RawEvent::LockFunds(target, amount, now));
            Ok(())
        }

        pub fn unlock_funds(origin, amount: BalanceOf<T>) -> DispatchResult {
            let target = ensure_signed(origin)?;

            T::Currency::unreserve(&target, amount);

            let now = <system::Module<T>>::block_number();

            Self::deposit_event(RawEvent::UnlockFunds(target, amount, now));
            Ok(())
        }

        pub fn transfer_funds(origin, dest: T::AcountId, amount: BalanceOf<T>) -> DispatchResult {
            let sender = ensure_signed(origin)?;

            T::Currency::transfer(&sender, &dest, amount);

            let now = <system::Module<T>>::block_number();

            Self::deposit_event(RawEvent::TransferFunds(sender, dest, amount, now));
            Ok(())
        }
    }
}

decl_error! {
    pub enum Error for Module<T: Trait> {
        AmbitiousReserve,
        AmbitiousTransfer,
        Unexplained
    }
}
