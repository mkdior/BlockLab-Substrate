// Auto-generated via `yarn polkadot-types-from-defs`, do not edit
/* eslint-disable */

import { ITuple } from '@polkadot/types/types';
import { Option, Struct } from '@polkadot/types/codec';
import { bool, u128, u32, u64 } from '@polkadot/types/primitive';
import { AccountId, H256 } from '@polkadot/types/interfaces/runtime';

/** @name Address */
export interface Address extends AccountId {}

/** @name AuctionCoreInfo */
export interface AuctionCoreInfo extends Struct {
  readonly timestamp: GeneralInformationContainer;
  readonly cargo: ITuple<[GeneralInformationContainer, GeneralInformationContainer]>;
}

/** @name AuctionId */
export interface AuctionId extends u64 {}

/** @name AuctionInfo */
export interface AuctionInfo extends Struct {
  readonly creator: AccountId;
  readonly slot_origin: AccountId;
  readonly bid: Option<ITuple<[AccountId, Balance]>>;
  readonly core: AuctionCoreInfo;
  readonly start: BlockNumber;
  readonly end: Option<BlockNumber>;
}

/** @name Balance */
export interface Balance extends u128 {}

/** @name BalanceOf */
export interface BalanceOf extends Balance {}

/** @name BlockNumber */
export interface BlockNumber extends u32 {}

/** @name Currency */
export interface Currency extends Balance {}

/** @name GeneralInformationContainer */
export interface GeneralInformationContainer extends u64 {}

/** @name GenInfo */
export interface GenInfo extends GeneralInformationContainer {}

/** @name Hash */
export interface Hash extends H256 {}

/** @name LookupSource */
export interface LookupSource extends AccountId {}

/** @name OnNewBidResult */
export interface OnNewBidResult extends Struct {
  readonly accept_bid: bool;
  readonly auction_end: Option<Option<BlockNumber>>;
}

/** @name QueuedBid */
export interface QueuedBid extends Struct {
  readonly bid: ITuple<[AccountId, Balance]>;
  readonly auction_id: AuctionId;
}

/** @name Status */
export interface Status extends bool {}

/** @name UIAuctionInfo */
export interface UIAuctionInfo extends Struct {
  readonly slot_owner: AccountId;
  readonly slot_origin: AccountId;
  readonly slot_time: GeneralInformationContainer;
  readonly slot_num_cargo: GeneralInformationContainer;
  readonly slot_num_teu: GeneralInformationContainer;
  readonly auction_is_live: bool;
  readonly auction_highest_bid: Option<ITuple<[AccountId, Balance]>>;
  readonly auction_end_time: Option<BlockNumber>;
}

export type PHANTOM_AUCTIONS = 'auctions';
