use auction_api::AuctionInformationAPI as AuctionRuntimeAPI;
use jsonrpc_core::{Error as RpcError, ErrorCode, Result};
use jsonrpc_derive::rpc;
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_runtime::{generic::BlockId, traits::Block as BlockT};
use std::sync::Arc;

#[rpc]
pub trait AuctionInformationAPI<BlockHash, AccountId, AuctionId, Balance, BlockNumber, GeneralInfo>
{
    #[rpc(name = "auctionInformation_exists")]
    fn auction_exists(&self, at: Option<BlockHash>, id: AuctionId) -> Result<bool>;
    #[rpc(name = "auctionInformation_queryInformal")]
    fn auction_query_informal(
        &self,
        at: Option<BlockHash>,
        id: AuctionId,
    ) -> Result<Option<AuctionInfo<AccountId, Balance, BlockNumber, GeneralInfo>>>;
    #[rpc(name = "auctionInformation_queryInformalAll")]
    fn auction_query_informal_all(
        &self,
        at: Option<BlockHash>,
    ) -> Result<Option<Vec<AuctionInfo<AccountId, Balance, BlockNumber, GeneralInfo>>>>;
    #[rpc(name = "auctionInformation_queryInformalAllStatus")]
    fn auction_query_informal_all_status(
        &self,
        at: Option<BlockHash>,
        active: bool,
    ) -> Result<Option<Vec<AuctionInfo<AccountId, Balance, BlockNumber, GeneralInfo>>>>;
    #[rpc(name = "auctionInformation_queryFormal")]
    fn auction_query_formal(
        &self,
        at: Option<BlockHash>,
        id: AuctionId,
    ) -> Result<Option<UIAuctionInfo<AccountId, Balance, BlockNumber, GeneralInfo>>>;
    #[rpc(name = "auctionInformation_queryFormalAll")]
    fn auction_query_formal_all(
        &self,
        at: Option<BlockHash>,
    ) -> Result<Option<Vec<UIAuctionInfo<AccountId, Balance, BlockNumber, GeneralInfo>>>>;
    #[rpc(name = "auctionInformation_queryFormalAllStatus")]
    fn auction_query_formal_all_status(
        &self,
        at: Option<BlockHash>,
        active: bool,
    ) -> Result<Option<Vec<UIAuctionInfo<AccountId, Balance, BlockNumber, GeneralInfo>>>>;
}

pub struct AuctionInformation<C, M> {
    client: Arc<C>,
    _marker: std::marker::PhantomData<M>,
}

impl<C, M> AuctionInformation<C, M> {
    pub fn new(client: Arc<C>) -> Self {
        Self {
            client,
            _marker: Default::default(),
        }
    }
}


// E: AccountId, F: AuctionId, G: Balance, H: BlockNumber, I: GeneralInfo>
impl<C, Block, E, F, G, H, I> AuctionInformationAPI<<Block as BlockT>::Hash> for AuctionInformation<C, Block>
where
	Block: BlockT,
	C: Send + Sync + 'static,
	C: ProvideRuntimeApi<Block>,
	C: HeaderBackend<Block>,
	C::Api: SumStorageRuntimeApi<Block>,
        <++> CONT
{
	fn get_sum(&self, at: Option<<Block as BlockT>::Hash>) -> Result<u32> {
		let api = self.client.runtime_api();
		let at = BlockId::hash(at.unwrap_or_else(||
			// If the block hash is not supplied assume the best block.
			self.client.info().best_hash));

		let runtime_api_result = api.get_sum(&at);
		runtime_api_result.map_err(|e| RpcError {
			code: ErrorCode::ServerError(9876), // No real reason for this value
			message: "Something wrong".into(),
			data: Some(format!("{:?}", e).into()),
		})
	}
}
