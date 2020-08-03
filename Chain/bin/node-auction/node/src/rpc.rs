use std::{fmt, sync::Arc};

use node_auction_runtime::{
    opaque::Block, AccountId, AuctionId, Balance, GeneralInformationContainer, Hash, Index,
    UncheckedExtrinsic,
};
use sc_client_api::backend::{Backend, StateBackend, StorageProvider};
use sc_rpc_api::DenyUnsafe;
use sp_api::ProvideRuntimeApi;
use sp_block_builder::BlockBuilder;
use sp_blockchain::{Error as BlockChainError, HeaderBackend, HeaderMetadata};
use sp_consensus::SelectChain;
use sp_runtime::traits::BlakeTwo256;
use sp_transaction_pool::TransactionPool;

// NOTE: Allow for now, perhaps this'll be used upon cleanup.
#[allow(dead_code)]
pub type IoHandler = jsonrpc_core::IoHandler<sc_rpc::Metadata>;

/// Light client extra dependencies.
pub struct LightDeps<C, F, P> {
    /// The client instance to use.
    pub client: Arc<C>,
    /// Transaction pool instance.
    pub pool: Arc<P>,
    /// Remote access to the blockchain (async).
    pub remote_blockchain: Arc<dyn sc_client_api::light::RemoteBlockchain<Block>>,
    /// Fetcher instance.
    pub fetcher: Arc<F>,
}

/// Full client dependencies.
pub struct FullDeps<C, P, SC> {
    /// The client instance to use.
    pub client: Arc<C>,
    /// Transaction pool instance.
    pub pool: Arc<P>,
    /// The SelectChain Strategy
    pub select_chain: SC,
    /// Whether to deny unsafe calls
    pub deny_unsafe: DenyUnsafe,
    /// The Node authority flag
    pub is_authority: bool,
}

/// Instantiate all Full RPC extensions.
pub fn create_full<C, P, SC, BE>(
    deps: FullDeps<C, P, SC>,
) -> jsonrpc_core::IoHandler<sc_rpc::Metadata>
where
    BE: Backend<Block> + 'static,
    BE::State: StateBackend<BlakeTwo256>,
    C: ProvideRuntimeApi<Block> + StorageProvider<Block, BE>,
    C: HeaderBackend<Block> + HeaderMetadata<Block, Error = BlockChainError> + 'static,
    C: Send + Sync + 'static,
    C::Api: system_rpc::AccountNonceApi<Block, AccountId, Index>,
    C::Api: BlockBuilder<Block>,
    C::Api: payment_rpc::TransactionPaymentRuntimeApi<Block, Balance, UncheckedExtrinsic>,
    C::Api: auction_api::AuctionInformationAPI<Block, AuctionId>,
    <C::Api as sp_api::ApiErrorExt>::Error: fmt::Debug,
    P: TransactionPool<Block = Block> + 'static,
    //	M: jsonrpc_core::Metadata + Default,
    SC: SelectChain<Block> + 'static,
{
    use payment_rpc::{TransactionPayment, TransactionPaymentApi};
    use system_rpc::{FullSystem, SystemApi};

    let mut io = jsonrpc_core::IoHandler::default();

    let FullDeps {
        client,
        pool,
        select_chain,
        deny_unsafe,
        is_authority,
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

    //AuctionAPI definition::
    //AuctionInformationAPI<AccountId, AuctionId, Balance, BlockNumber, GeneralInfo>

    io
}

/// Instantiate all Light RPC extensions.
pub fn create_light<C, P, M, F>(deps: LightDeps<C, F, P>) -> jsonrpc_core::IoHandler<M>
where
    C: sp_blockchain::HeaderBackend<Block>,
    C: Send + Sync + 'static,
    F: sc_client_api::light::Fetcher<Block> + 'static,
    P: TransactionPool + 'static,
    M: jsonrpc_core::Metadata + Default,
{
    use system_rpc::{LightSystem, SystemApi};

    let LightDeps {
        client,
        pool,
        remote_blockchain,
        fetcher,
    } = deps;
    let mut io = jsonrpc_core::IoHandler::default();
    io.extend_with(SystemApi::<Hash, AccountId, Index>::to_delegate(
        LightSystem::new(client, remote_blockchain, fetcher, pool),
    ));

    io
}
