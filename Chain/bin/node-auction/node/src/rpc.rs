use std::{fmt, sync::Arc};

use node_auction_runtime::{
    opaque::Block, AccountId, AuctionId, Balance, BlockNumber, GeneralInformationContainer, Hash,
    Index, UncheckedExtrinsic,
};
use sc_client_api::backend::{Backend, StateBackend, StorageProvider};
use sc_rpc_api::DenyUnsafe;
use sp_api::ProvideRuntimeApi;
use sp_block_builder::BlockBuilder;
use sp_blockchain::{Error as BlockChainError, HeaderBackend, HeaderMetadata};
use sp_consensus::SelectChain;
use sp_runtime::traits::BlakeTwo256;
use sp_transaction_pool::TransactionPool;

/// Full client dependencies.
pub struct FullDeps<C, P> {
    /// The client instance to use.
    pub client: Arc<C>,
    /// Transaction pool instance.
    pub pool: Arc<P>,
    /// Whether to deny unsafe calls
    pub deny_unsafe: DenyUnsafe,
}

/// Instantiate all Full RPC extensions.
pub fn create_full<C, P, BE>(deps: FullDeps<C, P>) -> jsonrpc_core::IoHandler<sc_rpc::Metadata>
where
    BE: Backend<Block> + 'static,
    BE::State: StateBackend<BlakeTwo256>,
    C: ProvideRuntimeApi<Block> + StorageProvider<Block, BE>,
    C: HeaderBackend<Block> + HeaderMetadata<Block, Error = BlockChainError> + 'static,
    C: Send + Sync + 'static,
    C::Api: BlockBuilder<Block>,
    C::Api: system_rpc::AccountNonceApi<Block, AccountId, Index>,
    C::Api: payment_rpc::TransactionPaymentRuntimeApi<Block, Balance, UncheckedExtrinsic>,
    C::Api: auction_api::AuctionInformationAPI<
        Block,
        AccountId,
        AuctionId,
        Balance,
        BlockNumber,
        GeneralInformationContainer,
    >,
    <C::Api as sp_api::ApiErrorExt>::Error: fmt::Debug,
    P: TransactionPool<Block = Block> + 'static,
{
    use payment_rpc::{TransactionPayment, TransactionPaymentApi};
    use system_rpc::{FullSystem, SystemApi};

    let mut io = jsonrpc_core::IoHandler::default();

    let FullDeps {
        client,
        pool,
        deny_unsafe,
    } = deps;

    io.extend_with(SystemApi::to_delegate(FullSystem::new(
        client.clone(),
        pool.clone(),
        deny_unsafe,
    )));
    io.extend_with(TransactionPaymentApi::to_delegate(TransactionPayment::new(
        client.clone(),
    )));
    io.extend_with(auction_rpc::AuctionInformationAPI::to_delegate(
        auction_rpc::AuctionInformation::new(client.clone()),
    ));

    io
}
