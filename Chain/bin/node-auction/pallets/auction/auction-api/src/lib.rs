#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::too_many_arguments)]

use auction_traits::auction::{UIAuctionInfo};
use frame_support::inherent::Vec;
use parity_scale_codec::Codec;
sp_api::decl_runtime_apis! {
    pub trait AuctionInformationAPI<AccountId, AuctionId, Balance, BlockNumber, GeneralInfo>
    where
        AccountId: Codec,
        AuctionId: Codec,
        Balance: Codec,
        BlockNumber: Codec,
        GeneralInfo: Codec,
    {
        fn auction_exists(id: AuctionId) -> bool;
        fn auction_query_informal(
            id: AuctionId,
        ) -> Option<AuctionInfo<AccountId, Balance, BlockNumber, GeneralInfo>>;
        fn auction_query_informal_all(
        ) -> Option<Vec<AuctionInfo<AccountId, Balance, BlockNumber, GeneralInfo>>>;
        fn auction_query_informal_all_status(
            active: bool,
        ) -> Option<Vec<AuctionInfo<AccountId, Balance, BlockNumber, GeneralInfo>>>;
        fn auction_query_formal(
            id: AuctionId,
        ) -> Option<UIAuctionInfo<AccountId, BlockNumber, GeneralInfo>>;
        fn auction_query_formal_all(
        ) -> Option<Vec<UIAuctionInfo<AccountId, Balance, BlockNumber, GeneralInfo>>>;
        fn auction_query_formal_all_status(
            active: bool,
        ) -> Option<Vec<UIAuctionInfo<AccountId, Balance, BlockNumber, GeneralInfo>>>;
    }
}
