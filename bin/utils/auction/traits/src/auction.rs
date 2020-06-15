use codec::FullCodec;
use codec::{Decode, Encode};
use sp_runtime::{traits::MaybeSerializeDeserialize, DispatchResult, RuntimeDebug};
use sp_std::{
    cmp::{Eq, PartialEq},
    fmt::Debug,
};
/// Queued bid information.
#[cfg_attr(feature = "std", derive(PartialEq, Eq))]
#[derive(Encode, Decode, RuntimeDebug, Clone, Copy)]
pub struct QueuedBid<AccountId, Balance, AuctionId> {
    /// The bid to be queued.
    pub bid: (AccountId, Balance),
    /// The auction this bid is destined for.
    pub auction_id: AuctionId,
}

/// Information being sold in the auction, in our case the actual time-slot. For now all we're
/// storing is the timestamp and the cargo information in a tuple of (number of containers, TEUs). For TEUs we're assuming (1TEU==1Container).
/// The timestamp is stored in UNIX format :: https://en.wikipedia.org/wiki/Unix_time
#[cfg_attr(feature = "std", derive(PartialEq, Eq))]
#[derive(Encode, Decode, RuntimeDebug, Clone, Copy)]
pub struct AuctionCoreInfo {
    pub timestamp: u64,
    pub cargo: (i32, i32),
}

/// Auction information. The creator of the auction is always the barge. Upon creating the auction,
/// the barge also states which terminal this auctioned off slot belongs to. This can later be
/// expanded into verification of slot ownership etc.
#[cfg_attr(feature = "std", derive(PartialEq, Eq))]
#[derive(Encode, Decode, RuntimeDebug, Clone, Copy)]
pub struct AuctionInfo<AccountId, Balance, BlockNumber> {
    /// Creator of the auction (Barge)
    pub creator: AccountId,
    /// Owner of the initially issued slot (Terminal)
    pub slot_origin: AccountId,
    /// Current bidder and bid price.
    pub bid: Option<(AccountId, Balance)>,
    /// Core auction information
    pub core: AuctionCoreInfo,
    /// Define which block this auction will be started.
    pub start: BlockNumber,
    /// Define which block this auction will be ended.
    pub end: Option<BlockNumber>,
}

/// Abstraction over a simple auction system.
pub trait Auction<AccountId, BlockNumber> {
    /// The id of an AuctionInfo
    type AuctionId: FullCodec + Default + Copy + Eq + PartialEq + MaybeSerializeDeserialize + Debug;
    /// The price to bid.
    type Balance: Encode + Decode + Default + Clone + PartialEq;

    /// The auction info of `id`
    fn auction_info(
        id: Self::AuctionId,
    ) -> Option<AuctionInfo<AccountId, Self::Balance, BlockNumber>>;
    /// Update the auction info of `id` with `info`
    fn update_auction(
        id: Self::AuctionId,
        info: AuctionInfo<AccountId, Self::Balance, BlockNumber>,
    ) -> DispatchResult;
    /// Create new auction with specific startblock and endblock, return the id of the auction
    fn new_auction(
        barge: AccountId,
        terminal: AccountId,
        core_info: AuctionCoreInfo,
        start: BlockNumber,
        end: Option<BlockNumber>,
    ) -> Self::AuctionId;
    /// Remove auction by `id`
    fn remove_auction(id: Self::AuctionId);
}

/// The result of bid handling.
pub struct OnNewBidResult<BlockNumber> {
    /// Indicates if the bid was accepted
    pub accept_bid: bool,
    /// `None` means don't change, `Some(None)` means no more auction end time, `Some(Some(number))` means set auction end time to this block
    pub auction_end: Option<Option<BlockNumber>>,
}

/// Hooks for auction to handle bids.
pub trait AuctionHandler<AccountId, Balance, BlockNumber, AuctionId> {
    /// Called when new bid is received.
    /// The return value deteermine if the bid should be accepted and update auction end time.
    /// Implementation should reserve money from current winner and refund previous winner.
    fn on_new_bid(
        now: BlockNumber,
        id: AuctionId,
        new_bid: (AccountId, Balance),
        last_bid: Option<(AccountId, Balance)>,
    ) -> OnNewBidResult<BlockNumber>;
    /// End an auction with `winner`
    fn on_auction_ended(
        id: AuctionId,
        recipients: (AccountId, AccountId),
        winner: Option<(AccountId, Balance)>,
    );
}
