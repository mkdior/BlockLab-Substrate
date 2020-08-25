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

			},
            queryFormal: {

			},
            queryFormalAll: {

			},
            queryFormalAllStatus: {

			},
            queryInformal: {

			},
            queryInformalAll: {

			},
            queryInformalAllStatus: {

			}
        },
        system: {
            dryRun: {

			},
            dryRunAt: {

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
