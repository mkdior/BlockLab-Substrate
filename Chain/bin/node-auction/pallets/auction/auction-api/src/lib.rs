#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::too_many_arguments)]

use auction_traits::auction::{AuctionInfo, UIAuctionInfo};
use frame_support::inherent::*;
use frame_support::traits::*;

sp_api::decl_runtime_apis! {
    pub trait AuctionInformationAPI<AccountId, AuctionId, Balance, BlockNumber, GeneralInfo> {
        fn auction_exists(id: AuctionId) -> bool;
        fn auction_query_informal(
            id: AuctionId,
        ) -> Option<Box<AuctionInfo<AccountId, Balance, BlockNumber, GeneralInfo>>>;
        fn auction_query_informal_all(
        ) -> Option<Box<Vec<AuctionInfo<AccountId, Balance, BlockNumber, GeneralInfo>>>>;
        fn auction_query_informal_all_status(
            active: bool,
        ) -> Option<Box<Vec<AuctionInfo<AccountId, Balance, BlockNumber, GeneralInfo>>>>;
        fn auction_query_formal(
            id: AuctionId,
        ) -> Option<Box<UIAuctionInfo<AccountId, Balance, BlockNumber, GeneralInfo>>>;
        fn auction_query_formal_all(
        ) -> Option<Box<Vec<UIAuctionInfo<AccountId, Balance, BlockNumber, GeneralInfo>>>>;
        fn auction_query_formal_all_status(
            active: bool,
        ) -> Option<Box<Vec<UIAuctionInfo<AccountId, Balance, BlockNumber, GeneralInfo>>>>;
    }
}
