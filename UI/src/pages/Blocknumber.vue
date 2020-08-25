<template>
  <div class="content">
    <card>
      <template slot="header">
        <h5 class="title">Block Overview</h5>
        <p class="category">Current Block : </p>
      </template>
      <div class="row">
        <div class="col-lg-2 col-md-3 col-sm-4 col-xs-6 col-xs-6">
          {{ blocknumber }}
        </div>
      </div>
    </card>
  </div>
</template>
<script>
import { Card } from "@/components/index";

import api from "../connection.js";

// api.rpc.auctionInformation.[]

export default {
  components: {
    Card,
  },
  data() {
    return {
      blocknumber: 0,
      events: [],
    };
  },
  methods: {
    getBlockNumber() {
      api.then((api) => {
        api.rpc.chain.getBlock().then((block) => {
		console.log(api.rpc.auctionInformation.queryFormal("0x0f0bdccf8ee18903d198b0898220ea86ddb2bbcc2c94fef55de7d14a5c15e73e", 7));
          this.blocknumber = block.block.header.number.toNumber();
        });
      });
    },
  },
  created() {
    this.getBlockNumber();
  },
};
</script>
<style></style>
