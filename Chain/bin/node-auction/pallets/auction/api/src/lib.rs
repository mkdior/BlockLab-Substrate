#[allow(unused_imports)]
use auction_traits::auction::*;

#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::too_many_arguments)]

// Here we declare the runtime API. It is implemented it the `impl` block in
// runtime amalgamator file (the `runtime/src/lib.rs`)
sp_api::decl_runtime_apis! {
	pub trait AuctionAPI<AccountId, AuctionId, BlockNumber, Balance> {
                fn auction_exists(id: AuctionId) -> bool; 
                fn auction_query_informal(id: AuctionId) ->
	}
}
