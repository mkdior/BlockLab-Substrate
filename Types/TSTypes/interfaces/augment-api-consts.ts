// Auto-generated via `yarn polkadot-types-from-chain`, do not edit
/* eslint-disable */

import { Vec } from '@polkadot/types/codec';
import { u32 } from '@polkadot/types/primitive';
import { Balance, BalanceOf, BlockNumber, Moment, RuntimeDbWeight, Weight } from '@polkadot/types/interfaces/runtime';
import { WeightToFeeCoefficient } from '@polkadot/types/interfaces/support';

declare module '@polkadot/metadata/Decorated/consts/types' {
  export interface Constants {
    balances: {
      /**
       * The minimum amount required to keep an account open.
       **/
      existentialDeposit: AugmentedConst<Balance>;
    };
    system: {
      /**
       * The base weight of executing a block, independent of the transactions in the block.
       **/
      blockExecutionWeight: AugmentedConst<Weight>;
      /**
       * The maximum number of blocks to allow in mortal eras.
       **/
      blockHashCount: AugmentedConst<BlockNumber>;
      /**
       * The weight of runtime database operations the runtime can invoke.
       **/
      dbWeight: AugmentedConst<RuntimeDbWeight>;
      /**
       * The base weight of an Extrinsic in the block, independent of the of extrinsic being executed.
       **/
      extrinsicBaseWeight: AugmentedConst<Weight>;
      /**
       * The maximum length of a block (in bytes).
       **/
      maximumBlockLength: AugmentedConst<u32>;
      /**
       * The maximum weight of a block.
       **/
      maximumBlockWeight: AugmentedConst<Weight>;
    };
    timestamp: {
      /**
       * The minimum period between blocks. Beware that this is different to the *expected* period
       * that the block production apparatus provides. Your chosen consensus system will generally
       * work with this to determine a sensible block time. e.g. For Aura, it will be double this
       * period on default settings.
       **/
      minimumPeriod: AugmentedConst<Moment>;
    };
    transactionPayment: {
      /**
       * The fee to be paid for making a transaction; the per-byte portion.
       **/
      transactionByteFee: AugmentedConst<BalanceOf>;
      /**
       * The polynomial that is applied in order to derive fee from weight.
       **/
      weightToFee: AugmentedConst<Vec<WeightToFeeCoefficient>>;
    };
  }
}
