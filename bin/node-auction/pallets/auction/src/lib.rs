#![cfg_attr(not(feature = "std"), no_std)]

////////////////////////////////////////////////////////////////////////////////////////////////
// Imports
// /////////////////////////////////////////////////////////////////////////////////////////////
#[allow(unused_imports)]
use frame_support::{
    decl_error,
    decl_event,
    decl_module,
    decl_storage,
    dispatch::Parameter,
    ensure,
    sp_runtime::{
        traits::{AtLeast32Bit, MaybeSerializeDeserialize, Member, One, Zero},
        DispatchError, DispatchResult,
    },
    traits::{Currency, ExistenceRequirement::AllowDeath, ReservableCurrency},
    weights::SimpleDispatchInfo, //storage::IterableStorageDoubleMap,
    IterableStorageDoubleMap,
};

#[allow(unused_imports)]
use frame_system::{self as system, ensure_signed};

#[allow(unused_imports)]
use orml_traits::auction::{Auction, AuctionHandler, AuctionInfo};

use serde;

////////////////////////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests;

type BalanceOf<T> = <<T as Trait>::Currency as Currency<<T as system::Trait>::AccountId>>::Balance;

/// The pallet's configuration trait.
pub trait Trait: system::Trait + Sized {
    /// The overarching event type.
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
    type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;
    type AuctionId: Parameter + Member + AtLeast32Bit + Default + Copy + MaybeSerializeDeserialize;
    type Handler: AuctionHandler<
        Self::AccountId,
        BalanceOf<Self>,
        Self::BlockNumber,
        Self::AuctionId,
    >;
}

decl_storage! {
    trait Store for Module<T: Trait> as AuctionModule {
        pub Auctions get(fn auctions) config(): map hasher(twox_64_concat) T::AuctionId => Option<AuctionInfo<T::AccountId, BalanceOf<T>, T::BlockNumber>>;
        pub AuctionsIndex get(fn auctions_index): T::AuctionId;
        pub AuctionEndTime get(fn auction_end_time): double_map hasher(twox_64_concat) T::BlockNumber, hasher(twox_64_concat) T::AuctionId => Option<bool>;
}}

decl_event!(
    pub enum Event<T>
    where
        AccountId = <T as system::Trait>::AccountId,
        Balance = BalanceOf<T>,
        BlockNumber = <T as system::Trait>::BlockNumber,
        AuctionId = <T as Trait>::AuctionId,
    {
        LockFunds(AccountId, Balance, BlockNumber),
        UnlockFunds(AccountId, Balance, BlockNumber),
        TransferFunds(AccountId, AccountId, Balance, BlockNumber),
        Bid(AuctionId, AccountId, Balance),
    }
);

decl_module! {
   pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        type Error = Error<T>;

        fn deposit_event() = default;

        #[weight = SimpleDispatchInfo::FixedNormal(100)]
        pub fn bid(origin, id: T::AuctionId, #[compact] value: BalanceOf<T>) -> DispatchResult {
            let bidder = ensure_signed(origin)?;
            let mut auction = <Auctions<T>>::get(id).ok_or(Error::<T>::AuctionNotExist)?;
            let block_number = <frame_system::Module<T>>::block_number();

            ensure!(block_number >= auction.start, Error::<T>::AuctionNotStarted);

            //TODO(Hamza): Simplify this
            if let Some(ref current_bid) = auction.bid {
                ensure!(value > current_bid.1, Error::<T>::InvalidBidPrice);
            } else {
                ensure!(!value.is_zero(), Error::<T>::InvalidBidPrice);
            }

            //TODO(Hamza): Check source, function call might be wrong
            let bid_result = T::Handler::on_new_bid(block_number, id, (bidder.clone(), value), auction.bid.clone());

            ensure!(bid_result.accept_bid, Error::<T>::BidNotAccepted);
            
            // 
            if let Some(new_end) = bid_result.auction_end {
                if let Some(old_end_block) = auction.end {
                    <AuctionEndTime<T>>::remove(&old_end_block, id);
                }
                if let Some(new_end_block) = new_end {
                    <AuctionEndTime<T>>::insert(&new_end_block, id, true);
                }
                auction.end = new_end;
            }

            auction.bid = Some((bidder.clone(), value));
            <Auctions<T>>::insert(id, auction);
            Self::deposit_event(RawEvent::Bid(id, bidder, value));

            Ok(())
        }

        fn on_finalize(now: T::BlockNumber) {
            Self::_on_finalize(now);
        }
    }
}

decl_error! {
    pub enum Error for Module<T: Trait> {
        AuctionNotExist,
        AuctionNotStarted,
        BidNotAccepted,
        InvalidBidPrice,

        AmbitiousReserve,
        AmbitiousTransfer,
        Unexplained,
    }
}

impl<T: Trait> Module<T> {
    // TODO(Hamza):
    // https://github.com/substrate-developer-hub/recipes/blob/master/pallets/weights/src/lib.rs
    pub fn reserve_funds(target: T::AccountId, amount: BalanceOf<T>) -> DispatchResult {
        //TODO(Hamza): Serve proper errors. Also perhaps implement Currency for our local trait
        // to avoid the use of Currency::X
        T::Currency::reserve(&target, amount).map_err(|_| "Not able to reserve");

        let now = <system::Module<T>>::block_number();

        Self::deposit_event(RawEvent::LockFunds(target, amount, now));
        Ok(())
    }

    pub fn unreserve_funds(target: T::AccountId, amount: BalanceOf<T>) -> DispatchResult {
        T::Currency::unreserve(&target, amount);

        let now = <system::Module<T>>::block_number();

        Self::deposit_event(RawEvent::UnlockFunds(target, amount, now));
        Ok(())
    }

    pub fn transfer_funds(from: T::AccountId, to: T::AccountId, amount: BalanceOf<T>) -> DispatchResult {
        T::Currency::transfer(&from, &to, amount, AllowDeath)?;

        let now = <system::Module<T>>::block_number();

        Self::deposit_event(RawEvent::TransferFunds(from, to, amount, now));
        Ok(())
    }

    fn _on_finalize(now: T::BlockNumber) {
        for (auction_id, _) in <AuctionEndTime<T>>::drain_prefix(&now) {
            if let Some(auction) = <Auctions<T>>::take(&auction_id) {
                T::Handler::on_auction_ended(auction_id, auction.bid.clone());
            }
        }
    }
}

impl<T: Trait> Auction<T::AccountId, T::BlockNumber> for Module<T> {
    type AuctionId = T::AuctionId;
    type Balance = BalanceOf<T>;

    fn auction_info(
        id: Self::AuctionId,
    ) -> Option<AuctionInfo<T::AccountId, Self::Balance, T::BlockNumber>> {
        Self::auctions(id)
    }

    fn update_auction(
        id: Self::AuctionId,
        info: AuctionInfo<T::AccountId, Self::Balance, T::BlockNumber>,
    ) -> DispatchResult {
        let auction = <Auctions<T>>::get(id).ok_or(Error::<T>::AuctionNotExist)?;

        if let Some(old_end) = auction.end {
            <AuctionEndTime<T>>::remove(&old_end, id);
        }

        if let Some(new_end) = info.end {
            <AuctionEndTime<T>>::insert(&new_end, id, true);
        }

        <Auctions<T>>::insert(id, info);

        Ok(())
    }

    fn new_auction(start: T::BlockNumber, end: Option<T::BlockNumber>) -> Self::AuctionId {
        let auction = AuctionInfo {
            bid: None,
            start,
            end,
        };
        let auction_id = Self::auctions_index();
        <AuctionsIndex<T>>::mutate(|n| *n += Self::AuctionId::one());
        <Auctions<T>>::insert(auction_id, auction);
        if let Some(end_block) = end {
            <AuctionEndTime<T>>::insert(&end_block, auction_id, true);
        }

        auction_id
    }

    fn remove_auction(id: Self::AuctionId) {
        if let Some(auction) = <Auctions<T>>::take(&id) {
            if let Some(end_block) = auction.end {
                <AuctionEndTime<T>>::remove(&end_block, id);
            }
        }
    }
}
