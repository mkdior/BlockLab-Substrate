import {
    default_endpoint
} from "./config.js";

const endpoint = localStorage.getItem("endpoint");
if (!endpoint || typeof endpoint !== "string" || endpoint.indexOf("ws") !== 0) {
    localStorage.setItem("endpoint", default_endpoint);
}

const {
    ApiPromise,
    WsProvider
} = require("@polkadot/api");
const wsProvider = new WsProvider(localStorage.getItem("endpoint"));

let api = ApiPromise.create({
    provider: wsProvider,
    rpc: {
        auctionInformation: {
            exists: {
                description: 'This function can be used to check whether or not an auction exists.',
                params: [{
                        name: 'at',
                        type: 'Hash',
                        isOptional: true
                    },
                    {
                        name: 'id',
                        type: 'AuctionId',
                    }
                ],
                type: 'bool'
            },
            queryFormal: {
                description: 'This function tries a query to the auction storage, if an auction is found under id, it will return said auction in its formal format.',
                params: [{
                        name: 'at',
                        type: 'Hash',
                        isOptional: true
                    },
                    {
                        name: 'id',
                        type: 'AuctionId',
                    }
                ],
                type: 'Option<UIAuctionInfo<AccountId, BlockNumber, GeneralInfo>>'
            },
            queryFormalAll: {
                description: 'This function tries a full query to the auction storage, if n auctions are found, n auctions are returned in their formal formats.',
                params: [{
                    name: 'at',
                    type: 'Hash',
                    isOptional: true
                }],
                type: 'Option<Vec<UIAuctionInfo<AccountId, BlockNumber, GeneralInfo>>>'
            },
            queryFormalAllStatus: {
                description: 'This function tries a full query to the auction storage based on the auction\'s status, if n auctions are found with status y, n auctions are returned with their respective status in their formal formats.',
                params: [{
                        name: 'at',
                        type: 'Hash',
                        isOptional: true
                    },
                    {
                        name: 'active',
                        type: 'bool',
                    }
                ],
                type: 'Option<Vec<UIAuctionInfo<AccountId, BlockNumber, GeneralInfo>>>'
            },
            queryInformal: {
                description: 'This function tries a query to the auction storage, if an auction is found under id, it will return said auction in its informal format.',
                params: [{
                        name: 'at',
                        type: 'Hash',
                        isOptional: true
                    },
                    {
                        name: 'id',
                        type: 'AuctionId',
                    }
                ],
                type: 'Option<AuctionInfo<AccountId, Balance, BlockNumber, GeneralInfo>>'
            },
            queryInformalAll: {
                description: 'This function tries a full query to the auction storage, if n auctions are found, n auctions are returned in their informal formats.',
                params: [{
                    name: 'at',
                    type: 'Hash',
                    isOptional: true
                }, ],
                type: 'Option<Vec<AuctionInfo<AccountId, Balance, BlockNumber, GeneralInfo>>>'
            },
            queryInformalAllStatus: {
                description: 'This function tries a full query to the auction storage based on the auction\'s status, if n auctions are found with status y, n auctions are returned with their respective statuses in their informal formats..',
                params: [{
                        name: 'at',
                        type: 'Hash',
                        isOptional: true
                    },
                    {
                        name: 'active',
                        type: 'bool',
                    }
                ],
                type: 'Option<Vec<AuctionInfo<AccountId, Balance, BlockNumber, GeneralInfo>>>'
            }
        },
        //	#[rpc(name = "system_dryRun", alias("system_dryRunAt"))]
        //	fn dry_run(&self, extrinsic: Bytes, at: Option<BlockHash>) -> FutureResult<Bytes>;
        system: {
            dryRun: {
                description: 'ETC',
                params: [{
                    name: 'extrinsic',
                    type: 'Bytes'
                }, ],
                type: 'Balance'
            },
            dryRunAt: {
                description: 'ETC',
                params: [{
                        name: 'extrinsic',
                        type: 'Bytes'
                    },
                    {
                        name: 'at',
                        type: 'Hash',
                        isOptional: true
                    },

                ],
                type: 'Bytes'
            },
        }
    },
    types: {
        Address: "AccountId",
        LookupSource: "AccountId",
        AuctionId: "u64",
        Balance: "u128",
        BalanceOf: "Balance",
        GeneralInformationContainer: "u64",
        GenInfo: "GeneralInformationContainer",
        Status: "bool",
        Dummy: "u64",
        Currency: "Balance",
        BlockNumber: "u32",
        Hash: "H256",
    },
    QueuedBid: {
        bid: "(AccountId, Balance)",
        auction_id: "AuctionId"
    },
    AuctionCoreInfo: {
        timestamp: "GeneralInformationContainer",
        cargo: "(GeneralInformationContainer, GeneralInformationContainer)"
    },
    AuctionInfo: {
        creator: "AccountId",
        slot_origin: "AccountId",
        bid: "Option<(AccountId, Balance)>",
        core: "AuctionCoreInfo<GeneralInformationContainer>",
        start: "BlockNumber",
        end: "Option<BlockNumber>"
    },
    UIAuctionInfo: {
        slot_owner: "AccountId",
        slot_origin: "AccountId",
        slot_time: "GeneralInformationContainer",
        slot_num_cargo: "GeneralInformationContainer",
        slot_num_teu: "GeneralInformationContainer",
        auction_is_live: "bool",
        auction_highest_bid: "Option<(AccountId, Balance)>",
        auction_end_time: "Option<BlockNumber>"
    },
    OnNewBidResult: {
        accept_bid: "bool",
        auction_end: "Option<Option<BlockNumber>>"
    }
});

//const util = require("@polkadot/util");
export default api;
