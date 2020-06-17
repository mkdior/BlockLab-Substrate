#![cfg_attr(not(feature = "std"), no_std)]

////////////////////////////////////////////////////////////////////////////////////////////////
// Imports
// /////////////////////////////////////////////////////////////////////////////////////////////
#[allow(unused_imports)]
use frame_support::{
    decl_error, decl_event, decl_module, decl_storage,
    dispatch::Parameter,
    ensure,
    sp_runtime::{
        traits::{AtLeast32Bit, MaybeSerializeDeserialize, Member, One, Zero},
        DispatchError, DispatchResult,
    },
    traits::{
        Currency, ExistenceRequirement::AllowDeath, ReservableCurrency, WithdrawReason,
        WithdrawReasons,
    },
    weights::{DispatchInfo, Weight},
    IterableStorageDoubleMap, IterableStorageMap,
};

#[allow(unused_imports)]
use frame_system::{self as system, ensure_signed};

#[allow(unused_imports)]
use auction_traits::auction::{Auction, AuctionCoreInfo, AuctionHandler, AuctionInfo, QueuedBid};

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
        // pub Auction get(fn auctions) config() <-- requires you to set the initial values in
        // Genesis configured for this runtime
        pub Auctions get(fn auctions): map hasher(twox_64_concat) T::AuctionId => Option<AuctionInfo<T::AccountId, BalanceOf<T>, T::BlockNumber>>;
        pub AuctionsIndex get(fn auctions_index): T::AuctionId;
        pub AuctionEndTime get(fn auction_end_time): double_map hasher(twox_64_concat) T::BlockNumber, hasher(twox_64_concat) T::AuctionId => Option<bool>;

        pub QueuedBids get(fn queued_bids): map hasher(twox_64_concat) T::BlockNumber => Option<QueuedBid<T::AccountId, BalanceOf<T>, T::AuctionId>>;
    }
        add_extra_genesis {
                               //        Barge       Terminal                          Start            End
            config(_auctions): Vec<(T::AccountId, T::AccountId, AuctionCoreInfo, T::BlockNumber, T::BlockNumber)>;

            build(|config: &GenesisConfig<T>| {
                for (barge, terminal, core_info, start, end) in &config._auctions {
                    assert!(
                        *end > *start,
                        "Ending block has to be greater than the starting block",
                    );
                    assert!(
                        *barge != *terminal,
                        "Barge operator cannot be the terminal",
                    );
                }
                for &(ref barge, ref terminal, ref core_info, ref start, ref end) in config._auctions.iter() {
                    <Module<T>>::new_auction(barge.clone(), terminal.clone(), *core_info, *start, Some(*end));
                    }
            });
    }
}

decl_event!(
    pub enum Event<T>
    where
        AccountId = <T as system::Trait>::AccountId,
        Balance = BalanceOf<T>,
        BlockNumber = <T as system::Trait>::BlockNumber,
        AuctionId = <T as Trait>::AuctionId,
    {
        // Currency Events
        // Called when funds are reserved (e.g. when placing a bid)
        LockFunds(AccountId, Balance, BlockNumber),
        // Called when releasing reserved funds.
        UnlockFunds(AccountId, Balance, BlockNumber),
        // Called when transfering released funds.
        TransferFunds(AccountId, AccountId, Balance, BlockNumber),
        // Auction Events
        // Called when a bid is placed.
        Bid(AuctionId, AccountId, Balance),
        // Called when a bid is queued.
        BidQueued(QueuedBid<AccountId, Balance, AuctionId>),
        // Called when a queued bid is placed
        BidQueuedPlaced(AuctionId, AccountId, Balance),
        // Called when an auction ends with 1+ bids.
        AuctionEndDecided(AccountId, AuctionId),
        // Called when an auction ends with 0 bids.
        AuctionEndUndecided(AuctionId),
        // Other Events
        DummyEvent(),
    }
);

decl_error! {
    pub enum Error for Module<T: Trait> {
        AuctionNotExist,
        AuctionNotStarted,
        BidNotAccepted,
        InvalidBidPrice,

        TryReserve,
        AmbitiousReserve,
        AmbitiousUnreserve,
        AmbitiousTransfer,
        Unexplained,
    }
}

decl_module! {
   pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        type Error = Error<T>;

        fn deposit_event() = default;

        #[weight = 10_000]
        pub fn bid(origin, id: T::AuctionId, #[compact] value: BalanceOf<T>) -> DispatchResult {
            let bidder = ensure_signed(origin)?;
            let mut auction = <Auctions<T>>::get(id).ok_or(Error::<T>::AuctionNotExist)?;
            let block_number = <frame_system::Module<T>>::block_number();

            // Queue bid if needed and exit.
            if block_number < auction.start {
                // We're placing a queued bid.
                // Check and see if current auction already has queued bids.
                // TODO(Hamza):: Current queued auctions are just linked to block_numbers, while
                // searching just through a block_number isn't that bad, perhaps try and actually
                // have the auction ID's exposed through a double map, this way we can search
                // focusing just on the auction ID, which saves us a loop.
                for (bnum, qbid) in <QueuedBids<T>>::iter() {
                    println!("Queued bid under block number: {} :: {:#?}", bnum, qbid);
                }

                // Assemble our queued bid.
                let queued_bid = QueuedBid {
                    bid: (bidder, value),
                    auction_id: id,
                };

                println!("{:#?} being inserted -- queued bid.", queued_bid);

                // Currently bids are queued regardless of the iniatiator's balance. This enables
                // bidders to cancel each-other's bids out by bidding something higher than the
                // other. To stop this from happening, we reserve some of the balance now.
                Self::reserve_funds(&queued_bid.bid.0, queued_bid.bid.1);
                // Note that the reserved balance isn't an exlusive pool of funds, other methods in
                // this runtime can pull from it. Perhaps something else is needed to make sure
                // that the unreserving funds are always successful.
                <QueuedBids<T>>::insert(auction.start, queued_bid.clone());
                Self::deposit_event(RawEvent::BidQueued(queued_bid));
                return Ok(());
            }

            if let Some(ref current_bid) = auction.bid {
                ensure!(value > current_bid.1, Error::<T>::InvalidBidPrice);
            } else {
                ensure!(!value.is_zero(), Error::<T>::InvalidBidPrice);
            }

            let bid_result = T::Handler::on_new_bid(block_number, id, (bidder.clone(), value), auction.bid.clone());

            ensure!(bid_result.accept_bid, Error::<T>::BidNotAccepted);

            // Bid was accepted, time to refund the previous bidder.
            if let Some((a,b)) = &auction.bid {
                let (previous_bidder,previous_bid) = &auction.bid.unwrap();
                Self::unreserve_funds(previous_bidder, *previous_bid);
            } else {
                // If needed, add additional handling here for when there's no previous bid.
                println!("No previous bid, let's not unreserve the unreservable.");
            }

            // In case we're expecting a new end_time, replace it. This in essence extends the
            // auction.
            if let Some(new_end) = bid_result.auction_end {
                if let Some(old_end_block) = auction.end {
                    <AuctionEndTime<T>>::remove(&old_end_block, id);
                }
                if let Some(new_end_block) = new_end {
                    <AuctionEndTime<T>>::insert(&new_end_block, id, true);
                }
                auction.end = new_end;
            }

            // Reserve auction funds
            Self::reserve_funds(&bidder, value);
            // Update auction bid
            auction.bid = Some((bidder.clone(), value));
            <Auctions<T>>::insert(id, auction);
            // Emit bidding event
            Self::deposit_event(RawEvent::Bid(id, bidder, value));

            Ok(())
        }

        fn on_initialize(now: T::BlockNumber) -> Weight {
            Self::_on_initialize(now);

            0
        }

        fn on_finalize(now: T::BlockNumber) {
            Self::_on_finalize(now);
        }
    }
}

impl<T: Trait> Module<T> {
    // TODO(Hamza):
    // https://github.com/substrate-developer-hub/recipes/blob/master/pallets/weights/src/lib.rs
    pub fn reserve_funds(target: &T::AccountId, amount: BalanceOf<T>) -> DispatchResult {
        //TODO(Hamza): Serve proper errors. Also perhaps implement Currency for our local trait
        // to avoid the use of Currency::X
        let now = <system::Module<T>>::block_number();
        // Make sure that the amount to be reserved isn't higher than the actual balance of the
        // user trying to bid on the auction.
        ensure!(
            (T::Currency::free_balance(target) >= amount),
            <Error<T>>::AmbitiousReserve
        );
        // Ensure that the withdrawel can be made.
        T::Currency::ensure_can_withdraw(
            target,
            amount,
            WithdrawReason::Reserve.into(),
            T::Currency::free_balance(target) - amount,
        )
        .map_err(|_e| <Error<T>>::TryReserve)?;
        T::Currency::reserve(&target, amount);

        Ok(())
    }

    pub fn unreserve_funds(target: &T::AccountId, amount: BalanceOf<T>) -> DispatchResult {
        let now = <system::Module<T>>::block_number();
        T::Currency::unreserve(&target, amount);

        Ok(())
    }

    pub fn transfer_funds(
        from: &T::AccountId,
        to: &T::AccountId,
        amount: BalanceOf<T>,
    ) -> DispatchResult {
        // We're trying to make sure that the amount pulled from reserved is equal to the original
        // agreed upon reservation. If reserved is less than that, the amount due is stored in
        // overdraft.
        let overdraft = T::Currency::unreserve(from, amount);
        // If the overdraft is larger than zero, it means that the reserved balance has been
        // siphoned from by some other process, this is not meant to happen so stop the transfer if
        // it does happen.
        ensure!(!(overdraft > Zero::zero()), <Error<T>>::AmbitiousUnreserve);
        // If for some reason we get through the ensure with some balance in overdraft, make sure
        // to remove it from the total amount to at least get to the next checking phase where
        // it'll fail regardless, in a more elgant manner.
        T::Currency::transfer(from, to, amount - overdraft, AllowDeath)?;

        let now = <system::Module<T>>::block_number();

        Ok(())
    }

    fn auction_exists(id: T::AuctionId) -> bool {
        <Auctions<T>>::contains_key(id)
    }

    /// Since queued bids are technically already placed, we're not dealing with an origin. Due to
    /// this fact we're implementing a second bidding function accepting only a bid, with an
    /// already signed origin.
    fn place_queued_bid(
        qbid: QueuedBid<T::AccountId, BalanceOf<T>, T::AuctionId>,
    ) -> DispatchResult {
        println!(
            "Queued bid passed to the bidder by the initialier: Block : {} :: Bid : {:?}",
            <frame_system::Module<T>>::block_number(),
            qbid
        );

        let mut auction = <Auctions<T>>::get(qbid.auction_id).ok_or(Error::<T>::AuctionNotExist)?;
        let block_number = <frame_system::Module<T>>::block_number();

        if let Some(ref current_bid) = auction.bid {
            ensure!(qbid.bid.1 > current_bid.1, Error::<T>::InvalidBidPrice);
        } else {
            ensure!(!qbid.bid.1.is_zero(), Error::<T>::InvalidBidPrice);
        }

        let bid_result = T::Handler::on_new_bid(
            block_number,
            qbid.auction_id,
            (qbid.bid.0.clone(), qbid.bid.1),
            auction.bid.clone(),
        );

        ensure!(bid_result.accept_bid, Error::<T>::BidNotAccepted);

        if let Some(new_end) = bid_result.auction_end {
            if let Some(old_end_block) = auction.end {
                <AuctionEndTime<T>>::remove(&old_end_block, qbid.auction_id);
            }
            if let Some(new_end_block) = new_end {
                <AuctionEndTime<T>>::insert(&new_end_block, qbid.auction_id, true);
            }
            auction.end = new_end;
        }

        // Update the auction's bid with our queued bid.
        auction.bid = Some((qbid.bid.0.clone(), qbid.bid.1));
        println!(
            "Queued bid placed :: Updated auction information :: {:#?}",
            auction
        );
        <Auctions<T>>::insert(qbid.auction_id, auction);
        Self::deposit_event(RawEvent::BidQueuedPlaced(
            qbid.auction_id,
            qbid.bid.0,
            qbid.bid.1,
        ));

        Ok(())
    }

    fn _on_initialize(now: T::BlockNumber) {
        for qbid in <QueuedBids<T>>::take(&now) {
            println!(
                "Queued bid caught by the initializer : Block {} :: Bid {:?}",
                now, qbid
            );

            Self::place_queued_bid(qbid);
        }
    }

    fn _on_finalize(now: T::BlockNumber) {
        // Look and see if current BlockNumber is in AuctionEndTime
        //        println!("################# Block {} ##################", now);
        //        for auction in <Auctions<T>>::iter() {
        //            println!("Auction : {:?}", auction);
        //        }
        //        println!("#############################################");
        for (auction_id, _) in <AuctionEndTime<T>>::drain_prefix(&now) {
            // Drain_prefix removes all keys under the specified blocknumber
            if let Some(auction) = <Auctions<T>>::take(&auction_id) {
                println!("Current auction being finalized : {:?}", auction);
                T::Handler::on_auction_ended(
                    auction_id,
                    (auction.creator, auction.slot_origin),
                    auction.bid.clone(),
                );
            } else if let None = <Auctions<T>>::take(&auction_id) {
                // Auction_id not found, something went wrong here.
                println!("Something went wrong");
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
        println!("UPDATING auction");
        <Auctions<T>>::insert(id, info);

        Ok(())
    }

    fn new_auction(
        barge: T::AccountId,
        terminal: T::AccountId,
        core_info: AuctionCoreInfo,
        start: T::BlockNumber,
        end: Option<T::BlockNumber>,
    ) -> Self::AuctionId {
        let auction = AuctionInfo {
            creator: barge,
            slot_origin: terminal,
            bid: None,
            core: core_info,
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
