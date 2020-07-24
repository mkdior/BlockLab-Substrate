#![cfg_attr(not(feature = "std"), no_std)]

////////////////////////////////////////////
///////////////// Imports //////////////////
////////////////////////////////////////////

#[allow(unused_imports)]
use frame_support::{
    decl_error, decl_event, decl_module, decl_storage,
    dispatch::Parameter,
    ensure,
    sp_runtime::{
        traits::{AtLeast32Bit, MaybeSerializeDeserialize, Member, One, Printable, Zero},
        DispatchError, DispatchResult,
    },
    traits::{
        Currency, ExistenceRequirement::AllowDeath, ReservableCurrency, WithdrawReason,
        WithdrawReasons,
    },
    weights::{DispatchInfo, Weight},
    IterableStorageDoubleMap, IterableStorageMap,
};

// --TEMP--Hamza-- Vector definition comes from here; required!
use frame_support::inherent::*;

#[allow(unused_imports)]
use frame_system::{self as system, ensure_signed};

#[allow(unused_imports)]
use auction_traits::auction::*;

////////////////////////////////////////////
////////////////// Tests ///////////////////
////////////////////////////////////////////

#[cfg(test)]
mod tests;

////////////////////////////////////////////
///////// Auctioning Module Code ///////////
////////////////////////////////////////////

// The currency type used in this module.
pub type BalanceOf<T> =
    <<T as Trait>::Currency as Currency<<T as system::Trait>::AccountId>>::Balance;

// AuctionInfo condensed into a single type, note that type aliases cannot be used in the type
// definition list for the Events, Module and/or Storage structs/enum/traits.
pub type InfoCond<T> = AuctionInfo<
    <T as system::Trait>::AccountId,
    BalanceOf<T>,
    <T as system::Trait>::BlockNumber,
    <T as Trait>::GeneralInformationContainer,
>;

// Dummy type used for debugging purposes.
pub type Dummy = u64;

pub trait Trait: system::Trait + Sized {
    // The overarching event type.
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
    // Definition of this module's currency.
    type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;
    // Definition of this module's auction id number.
    type AuctionId: Parameter
        + Member
        + AtLeast32Bit
        + Default
        + Copy
        + MaybeSerializeDeserialize
        + Printable;
    // Definition of this module's general information type. This type makes sure that any
    // information stored in custom structs complies with Substrate's rules for stored types. Even
    // though native types are standardly forced into these bounds, this type has been added for
    // the sake of clarity.
    type GeneralInformationContainer: Parameter
        + Member
        + AtLeast32Bit
        + Default
        + Copy
        + MaybeSerializeDeserialize;
    // Definition of this module's handler type. The handler type is used to enforce "custom"
    // auctioning rules which are defined by the developer him/herself. If extra custom logic is
    // needed, this is the place to add it to.
    type Handler: AuctionHandler<
        Self::AccountId,
        BalanceOf<Self>,
        Self::BlockNumber,
        Self::AuctionId,
    >;
}

decl_storage! {
    trait Store for Module<T: Trait> as AuctionModule {
        // Storage for the auctions themselves. This is a mapping between an "AuctionId" and an
        // "AuctionInfo", the "AuctionInfo" is stored in an Option for ease of use.
        pub Auctions get(fn auctions): map hasher(twox_64_concat) T::AuctionId => Option<AuctionInfo<T::AccountId, BalanceOf<T>, T::BlockNumber, T::GeneralInformationContainer>>;
        // Storage for the number of auctions (processed and active) in this module. After an
        // auction has ended, this number is not reset.
        pub AuctionsIndex get(fn auctions_index): T::AuctionId;
        // Each block, this storage item is queued with the current block_number, if it's
        // contained within, it means that an auction is ending on this given block. This improves
        // lookup performance.
        pub AuctionEndTime get(fn auction_end_time): double_map hasher(twox_64_concat) T::BlockNumber, hasher(twox_64_concat) T::AuctionId => Option<bool>;
        // Storage for queued bids. Bids live in this item untill they can be placed, once placed,
        // they're removed from this item.
        pub QueuedBids get(fn queued_bids): map hasher(twox_64_concat) T::BlockNumber => Option<QueuedBid<T::AccountId, BalanceOf<T>, T::AuctionId>>;
    }
        // This section is used to process the genesis information provided into the auctioning
        // module. This is mainly needed for testing purposes.
        add_extra_genesis {
            config(_auctions):
                Vec<(T::AccountId,
                     T::AccountId,
                     Vec<T::GeneralInformationContainer>,
                     T::BlockNumber,
                     T::BlockNumber)>;

            build(|config: &GenesisConfig<T>| {
                for (barge, terminal, _core_info, start, end) in &config._auctions {
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
                    <Module<T>>::new_auction(barge.clone(), terminal.clone(), AuctionCoreInfo { timestamp: core_info[0], cargo: (core_info[1], core_info[2]), }, *start, Some(*end));
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
        GenInfo = <T as Trait>::GeneralInformationContainer,
    {
        // Currency Events
        // Called when funds are reserved (e.g. when placing a bid)
        LockFunds(AccountId, Balance, BlockNumber),
        // Called when releasing reserved funds.
        UnlockFunds(AccountId, Balance, BlockNumber),
        // Called when transfering released funds.
        TransferFunds(AccountId, AccountId, Balance, BlockNumber),

        /// Auction Events
        // Called when a bid is placed.
        Bid(AuctionId, AccountId, Balance),
        // Called when a bid is queued.
        BidQueued(AccountId, Balance, AuctionId),
        // Called when a queued bid is placed
        BidQueuedPlaced(AuctionId, AccountId, Balance),
        // Called when an auction ends with 1+ bids.
        AuctionEndDecided(AccountId, AuctionId),
        // Called when an auction ends with 0 bids.
        AuctionEndUndecided(AuctionId),
        // Called when a new auction is created.
        AuctionCreated(
            AuctionId,   // Auction ID
            AccountId,   // Creator
            AccountId,   // Terminal
            BlockNumber, // Start
            BlockNumber, // End
            GenInfo,     // Timestamp
            GenInfo,     // No. Containers
            GenInfo,     // No. in TEU
        ),
        // Called when an auction is updated.
        // TODO(HAMZA):: Reform this, perhaps event aren't meant to carry this much information?
        AuctionUpdated(
            // Old Auction Information
            AuctionId,   // Auction ID
            GenInfo,     // Timestamp
            GenInfo,     // No. Containers
            GenInfo,     // No. in TEU
            BlockNumber, // Start
            BlockNumber, // End
            // New Auction Information
            GenInfo,     // Timestamp
            GenInfo,     // No. Containers
            GenInfo,     // No. in TEU
            BlockNumber, // Start
            BlockNumber, // End
        ),
        // Called when an auction is deleted.
        AuctionDeleted(AuctionId),

        // Other Events
        DummyEvent(),
    }
);

decl_error! {
    pub enum Error for Module<T: Trait> {
        // Thrown when an auction which doesn't exist is called.
        AuctionNotExist,
        // Thrown when some action is performed on an auction which hasn't been started yet. To
        // perform said action, the auction has to be started.
        AuctionNotStarted,
        // Thrown when any given bid isn't accepted. This is mostly due to insufficient funds
        // available on the initiator's account.
        BidNotAccepted,
        // Thrown when any given initiator tries to bid an insane price.
        InvalidBidPrice,

        // Thrown when any given initiator tries to perform an action on some object which doesn't
        // belong to said initiator. This is usually thrown whenever an auction is updated/removed.
        PermissionError,
        // Thrown when any given initiator tries to update his/her auction start-time  while the auction is
        // already live.
        AuctionAlreadyLive,
        // Thrown when any given initiator tries to update his/her auction while the auction itself
        // already contains a bid. Once a user bids, the auction is set in stone, unless deleted.
        CannotUpdateActiveAuction,

        // Thrown when ensure_can_withdraw fails
        TryReserve,
        // Thrown when too much funds are being reserved. Initiator doesn't have enough funds.
        AmbitiousReserve,
        // Thrown when too much funds are being unreserved. This should never happen, if it does,
        // the whole chain should stop because there's a bug, hence the panic!.
        AmbitiousUnreserve,
        // Thrown when reserved funds are being overdrawn while transfering. This should never
        // happen, if it does, the whole chain should stop because there's a bug, hence the panic!.
        AmbitiousTransfer,

        // Thrown for testing purposes or when no explanation can be given.
        Unexplained,
    }
}

decl_module! {
   pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        type Error = Error<T>;

        fn deposit_event() = default;

        #[weight = 10_000]
        pub fn bid(origin, id: T::AuctionId, value: BalanceOf<T>) -> DispatchResult {
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
                //for (bnum, qbid) in <QueuedBids<T>>::iter() {
                    //println!("Queued bid under block number: {} :: {:#?}", bnum, qbid);
                //}

                // Assemble our queued bid.
                let queued_bid = QueuedBid {
                    bid: (bidder, value),
                    auction_id: id,
                };

                //println!("{:#?} being inserted -- queued bid.", queued_bid);

                // Currently bids are queued regardless of the iniatiator's balance. This enables
                // bidders to cancel each-other's bids out by bidding something higher than the
                // other. To stop this from happening, we reserve some of the balance now.
                let reserve_result = Self::reserve_funds(&queued_bid.bid.0, queued_bid.bid.1);

                if let Err(_) = reserve_result {
                    // Funds couldn't be reserved.
                    sp_runtime::print("ERROR -- BID(QUEUED) >> RESERVE_FUNDS");
                }
                // Note that the reserved balance isn't an exlusive pool of funds, other methods in
                // this runtime can pull from it. Perhaps something else is needed to make sure
                // that the unreserving funds are always successful.
                <QueuedBids<T>>::insert(auction.start, queued_bid.clone());
                Self::deposit_event(RawEvent::BidQueued(queued_bid.bid.0, queued_bid.bid.1, queued_bid.auction_id));
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
                let (previous_bidder,previous_bid) = (a, b);
                let unreserve_result = Self::unreserve_funds(previous_bidder, *previous_bid);
                if let Err(_) = unreserve_result {
                    // Funds couldn't be unreserved
                    sp_runtime::print("ERROR -- MODULE >> UNRESERVE_FUNDS >> RESERVE");
                }
            } else {
                // If needed, add additional handling here for when there's no previous bid.
                //println!("No previous bid, let's not unreserve the unreservable.");
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
            let reserve_result = Self::reserve_funds(&bidder, value);

            if let Err(_) = reserve_result {
                // Funds couldn't be reserved
                sp_runtime::print("ERROR -- MODULE >> RESERVE_FUNDS >> RESERVE");
            }

            // Update auction bid
            auction.bid = Some((bidder.clone(), value));
            <Auctions<T>>::insert(id, auction);
            // Emit bidding event
            Self::deposit_event(RawEvent::Bid(id, bidder, value));

            Ok(())
        }

        #[weight = 10_000]
        pub fn ext_update_auction(origin,
            id: T::AuctionId,
            time: Option<T::GeneralInformationContainer>,
            num_con: Option<T::GeneralInformationContainer>,
            num_teu: Option<T::GeneralInformationContainer>,
            start: Option<T::BlockNumber>,
            end: Option<T::BlockNumber>
        ) -> DispatchResult {
            let initiator = ensure_signed(origin)?;
            let event_info = <Module<T>>::update_auction(
                id,
                initiator,
                AuctionUpdateInfo {
                    timestamp: time,
                    num_con: num_con,
                    num_teu: num_teu,
                },
                start,
                end);

            if let Ok(info) = event_info {
                if let (Some(old_end), Some(new_end)) = (info.old.end, info.new.end) {
                    Self::deposit_event(
                        // Result<AuctionUpdateComplete<InfoCond<T>>, Error<T>
                        // Always true
                        RawEvent::AuctionUpdated(
                            id,
                            info.old.core.timestamp,
                            info.old.core.cargo.0,
                            info.old.core.cargo.1,
                            info.old.start,
                            old_end,

                            info.new.core.timestamp,
                            info.new.core.cargo.0,
                            info.new.core.cargo.1,
                            info.new.start,
                            new_end,
                            ));
                        }
            }
            Ok(())
    }

       #[weight = 10_000]
       pub fn ext_new_auction(
           origin,
           terminal: T::AccountId,
           num_con: T::GeneralInformationContainer,
           num_teu: T::GeneralInformationContainer,
           timestamp: T::GeneralInformationContainer,
           start: T::BlockNumber,
           end: T::BlockNumber,
       ) -> DispatchResult {
           let initiator = ensure_signed(origin)?;
           let (auction_id, auction) = <Module<T>>::new_auction(
               initiator,
               terminal,
               AuctionCoreInfo {
                   timestamp: timestamp,
                   cargo: (num_con, num_teu)
               },
               start,
               Some(end));
           if let Some(end) = auction.end {
               Self::deposit_event(
                   RawEvent::AuctionCreated(
                       auction_id,
                       auction.creator,
                       auction.slot_origin,
                       auction.start,
                       end,
                       auction.core.timestamp,
                       auction.core.cargo.0,
                       auction.core.cargo.1,
                       ));
           }

            Ok(())
       }

        #[weight = 10_000]
        pub fn ext_remove_auction(origin, id: T::AuctionId) -> DispatchResult {
            let initiator = ensure_signed(origin)?;
            let remove_auction = <Module<T>>::remove_auction(id, initiator);

            if let Ok(inner) = remove_auction {
                if let Some(_auction) = inner {
                    Self::deposit_event(RawEvent::AuctionDeleted(id));
                }
            }
            Ok(())
        }

        #[weight = 10_000]
        pub fn ext_dummy(og, cargo: Dummy) -> DispatchResult {
            let _origin = ensure_signed(og)?;
            sp_runtime::print("EXT -- DUMMY --");
            sp_runtime::print("EXT ## DUMMY ##");
            Ok(())
        }

        fn on_initialize(now: T::BlockNumber) -> Weight {
            // Logging for the runtime, for testin purposes only
            frame_support::print("HAMZA -- ##################################################");
            frame_support::print("--ACTIVE-- Autioning Pallet.");
            frame_support::print("HAMZA -- ##################################################");

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
        let _now = <system::Module<T>>::block_number();
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
        let reserve_result = T::Currency::reserve(&target, amount);

        if let Err(_) = reserve_result {
            // Even thought ensure_can_withdraw passed, something went wrong during reserve.
            sp_runtime::print("ERROR -- MODULE >> RESERVE_FUNDS >> RESERVE");
        }

        Ok(())
    }

    pub fn unreserve_funds(target: &T::AccountId, amount: BalanceOf<T>) -> DispatchResult {
        let _now = <system::Module<T>>::block_number();
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

        let _now = <system::Module<T>>::block_number();

        Ok(())
    }

    #[allow(dead_code)]
    pub fn auction_exists(id: T::AuctionId) -> bool {
        <Auctions<T>>::contains_key(id)
    }

    /// Returns an auction in its original form. Original means the format it's actually stored in the
    /// database.
    #[allow(dead_code)]
    pub fn auction_query_informal(
        id: T::AuctionId,
    ) -> Option<
        AuctionInfo<T::AccountId, BalanceOf<T>, T::BlockNumber, T::GeneralInformationContainer>,
    > {
        let auction = <Auctions<T>>::get(id);
        if auction.is_some() {
            auction
        } else {
            None
        }
    }

    /// Returns a vector of all currently stored auctions regardless of the auction's activity
    /// status. Returns everything in its original form.
    #[allow(dead_code)]
    pub fn auction_query_informal_all() -> Option<
        Vec<
            AuctionInfo<T::AccountId, BalanceOf<T>, T::BlockNumber, T::GeneralInformationContainer>,
        >,
    > {
        let query = <Auctions<T>>::iter().map(|x| x.1).collect::<Vec<_>>();
        if query.len() == 0 {
            None
        } else {
            Some(query)
        }
    }

    #[allow(dead_code)]
    pub fn auction_query_informal_all_status(
        _is_active: bool,
    ) -> Option<
        Vec<
            AuctionInfo<T::AccountId, BalanceOf<T>, T::BlockNumber, T::GeneralInformationContainer>,
        >,
    > {
        let query = <Auctions<T>>::iter()
            .filter(|x| {
                // x = (AuctionId -> AuctionInfo)
                x.1.start >= <frame_system::Module<T>>::block_number()
            })
            .map(|x| x.1)
            .collect::<Vec<_>>();

        if query.len() == 0 {
            None
        } else {
            Some(query)
        }
    }

    #[allow(dead_code)]
    pub fn auction_query_formal(
        id: T::AuctionId,
    ) -> Option<
        UIAuctionInfo<T::AccountId, BalanceOf<T>, T::BlockNumber, T::GeneralInformationContainer>,
    > {
        let auction = <Auctions<T>>::get(id);

        if let Some(_auction) = auction {
            Some(UIAuctionInfo {
                slot_owner: _auction.creator,
                slot_origin: _auction.slot_origin,
                slot_time: _auction.core.timestamp,
                slot_num_cargo: _auction.core.cargo.0,
                slot_num_teu: _auction.core.cargo.1,
                auction_is_live: (<frame_system::Module<T>>::block_number() >= _auction.start),
                auction_highest_bid: _auction.bid,
                auction_end_time: _auction.end,
            })
        } else {
            None
        }
    }

    #[allow(dead_code)]
    pub fn auction_query_formal_all() -> Option<
        Vec<
            UIAuctionInfo<
                T::AccountId,
                BalanceOf<T>,
                T::BlockNumber,
                T::GeneralInformationContainer,
            >,
        >,
    > {
        let query = <Auctions<T>>::iter()
            .map(|x| UIAuctionInfo {
                slot_owner: x.1.creator,
                slot_origin: x.1.slot_origin,
                slot_time: x.1.core.timestamp,
                slot_num_cargo: x.1.core.cargo.0,
                slot_num_teu: x.1.core.cargo.1,
                auction_is_live: (<frame_system::Module<T>>::block_number() >= x.1.start),
                auction_highest_bid: x.1.bid,
                auction_end_time: x.1.end,
            })
            .collect::<Vec<_>>();

        if query.len() == 0 {
            None
        } else {
            Some(query)
        }
    }

    #[allow(dead_code)]
    pub fn auction_query_formal_all_status(
        is_active: bool,
    ) -> Option<
        Vec<
            UIAuctionInfo<
                T::AccountId,
                BalanceOf<T>,
                T::BlockNumber,
                T::GeneralInformationContainer,
            >,
        >,
    > {
        let query = <Auctions<T>>::iter()
            .filter(|x| {
                // x = (AuctionId -> AuctionInfo)
                is_active != (x.1.start >= <frame_system::Module<T>>::block_number())
            })
            .map(|x| UIAuctionInfo {
                slot_owner: x.1.creator,
                slot_origin: x.1.slot_origin,
                slot_time: x.1.core.timestamp,
                slot_num_cargo: x.1.core.cargo.0,
                slot_num_teu: x.1.core.cargo.1,
                auction_is_live: is_active,
                auction_highest_bid: x.1.bid,
                auction_end_time: x.1.end,
            })
            .collect::<Vec<_>>();

        if query.len() == 0 {
            None
        } else {
            Some(query)
        }
    }

    /// Since queued bids are technically already placed, we're not dealing with an origin. Due to
    /// this fact we're implementing a second bidding function accepting only a bid, with an
    /// already signed origin.
    fn place_queued_bid(
        qbid: QueuedBid<T::AccountId, BalanceOf<T>, T::AuctionId>,
    ) -> DispatchResult {
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
        //println!(
        //    "Queued bid placed :: Updated auction information :: {:#?}",
        //    auction
        //);
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
            //println!(
            //    "Queued bid caught by the initializer : Block {} :: Bid {:?}",
            //    now, qbid
            //);

            let pq_result = Self::place_queued_bid(qbid);

            if let Err(_) = pq_result {
                // Something went wrong placing the queued bid, for now throw an event + log
                sp_runtime::print("ERROR -- _ON_INITIALIZE>>PLACE_QUEUED_BID ");
            }
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
                //println!("Current auction being finalized : {:?}", auction);
                T::Handler::on_auction_ended(
                    auction_id,
                    (auction.creator, auction.slot_origin),
                    auction.bid.clone(),
                );
            } else if let None = <Auctions<T>>::take(&auction_id) {
                // Auction_id not found, something went wrong here.
                //println!("Something went wrong");
            }
        }
    }

    // For testing purposes only, displays the type's underlying, core type by causing an error.
    // This function works as intended!.
    #[allow(dead_code)]
    fn type_disc(_t: T::GeneralInformationContainer, _b: T::AuctionId) {
        //let _t : () = t;
        //let _b : () = b;
    }
}

impl<T: Trait> Auction<T::AccountId, T::BlockNumber, T::GeneralInformationContainer, Error<T>>
    for Module<T>
{
    type AuctionId = T::AuctionId;
    type Balance = BalanceOf<T>;

    fn auction_info(
        id: Self::AuctionId,
    ) -> Option<
        AuctionInfo<T::AccountId, Self::Balance, T::BlockNumber, T::GeneralInformationContainer>,
    > {
        Self::auctions(id)
    }

    fn update_auction(
        id: Self::AuctionId,
        origin: T::AccountId,
        core_info: AuctionUpdateInfo<T::GeneralInformationContainer>,
        start: Option<T::BlockNumber>,
        end: Option<T::BlockNumber>,
    ) -> Result<AuctionUpdateComplete<InfoCond<T>>, Error<T>> {
        // Ensure auction exists and make it mutable for our adjustments
        let mut auction = <Auctions<T>>::get(id).ok_or(Error::<T>::AuctionNotExist)?;
        // Ensure that origin is the owner of the auction
        ensure!(auction.creator == origin, <Error<T>>::PermissionError);

        let auction_original = auction.clone();

        // Ensure that the auction has no bids, if it has bids it means that the buyer of the
        // auction expects a certain time-slot. If a bid has been placed, only cancellation is
        // possible.
        if let Some(_) = &auction.bid {
            return Err(<Error<T>>::CannotUpdateActiveAuction);
        }

        // Replace auction's end-time if specified by the origin
        if let Some(new_end) = end {
            if let Some(old_end) = auction.end {
                <AuctionEndTime<T>>::remove(&old_end, id);
            }
            <AuctionEndTime<T>>::insert(&new_end, id, true);
            auction.end = Some(new_end.clone());
        }

        // Replace auction's data if specified by the origin
        // Option<timestamp> -> Option<num_con> -> Option<num_teu>
        if let Some(timestamp) = core_info.timestamp {
            auction.core.timestamp = timestamp;
        }
        if let Some(num_con) = core_info.num_con {
            auction.core.cargo.0 = num_con;
        }
        if let Some(num_teu) = core_info.num_teu {
            auction.core.cargo.1 = num_teu;
        }

        if let Some(new_start) = start {
            // User wants to postpone the auction's start block.
            // First ensure that the auction hasn't already
            ensure!(
                auction.start > <system::Module<T>>::block_number(),
                <Error<T>>::AuctionAlreadyLive
            );
            // Ensure that propsed start hasn't passed already.
            ensure!(
                new_start > <system::Module<T>>::block_number(),
                <Error<T>>::AuctionAlreadyLive
            );

            // Update auction's start
            auction.start = new_start;
        }

        let auction_updated = auction.clone();

        <Auctions<T>>::insert(id, auction);
        Ok(AuctionUpdateComplete {
            old: auction_original,
            new: auction_updated,
        })
    }

    fn new_auction(
        barge: T::AccountId,
        terminal: T::AccountId,
        core_info: AuctionCoreInfo<T::GeneralInformationContainer>,
        start: T::BlockNumber,
        end: Option<T::BlockNumber>,
    ) -> (T::AuctionId, InfoCond<T>) {
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
        let new_auction_info = (auction_id, auction);
        <Auctions<T>>::insert(new_auction_info.0, new_auction_info.1.clone());
        if let Some(end_block) = end {
            <AuctionEndTime<T>>::insert(&end_block, auction_id, true);
        }

        new_auction_info
    }

    fn remove_auction(
        id: Self::AuctionId,
        origin: T::AccountId,
    ) -> Result<Option<InfoCond<T>>, Error<T>> {
        //TODO(HAMZA):: When placing our ensure within if let, we get an error stating that T, our
        //generic type for our error can't be sent safely across threads, inspect this.
        let _auction = <Auctions<T>>::get(id).ok_or(Error::<T>::AuctionNotExist)?;
        let _auction_inner: Option<InfoCond<T>>;
        ensure!(_auction.creator == origin, <Error<T>>::PermissionError);
        if let Some(auction) = <Auctions<T>>::take(&id) {
            _auction_inner = Some(auction.clone());
            if let Some(end_block) = auction.end {
                <AuctionEndTime<T>>::remove(&end_block, id);
            }
        } else {
            _auction_inner = None;
        }
        // Using option because returning not as an Option causes an error which would result in
        // much more bloat-code. Also because I have no idea how to instantiate a
        // GeneralInformationContainer type from nothing, it can't do conversions for some reason
        // even though it should be able to, look into it TODO(Hamza).
        Ok(_auction_inner)
    }
}
