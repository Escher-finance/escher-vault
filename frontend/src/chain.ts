// Minimal Babylon testnet chain configuration for Keplr
// Values sourced from project's config files

export const BABYLON = {
  chainId: "bbn-test-5",
  chainName: "Babylon Testnet",
  rpc: "https://babylon-testnet-rpc.polkachu.com",
  rest: "https://babylon-testnet-api.polkachu.com",
  bip44: { coinType: 118 },
  bech32Config: {
    bech32PrefixAccAddr: "bbn",
    bech32PrefixAccPub: "bbnpub",
    bech32PrefixValAddr: "bbnvaloper",
    bech32PrefixValPub: "bbnvaloperpub",
    bech32PrefixConsAddr: "bbnvalcons",
    bech32PrefixConsPub: "bbnvalconspub",
  },
  stakeCurrency: {
    coinDenom: "BBN",
    coinMinimalDenom: "ubbn",
    coinDecimals: 6,
  },
  currencies: [
    {
      coinDenom: "BBN",
      coinMinimalDenom: "ubbn",
      coinDecimals: 6,
    },
  ],
  feeCurrencies: [
    {
      coinDenom: "BBN",
      coinMinimalDenom: "ubbn",
      coinDecimals: 6,
      gasPriceStep: { low: 0.01, average: 0.025, high: 0.04 },
    },
  ],
  features: ["cosmwasm"],
  explorer: {
    base: "https://testnet.mintscan.io/babylon",
    tx: (hash: string) => `https://testnet.mintscan.io/babylon/txs/${hash}`,
    address: (addr: string) => `https://testnet.mintscan.io/babylon/account/${addr}`,
    contract: (addr: string) => `https://testnet.mintscan.io/babylon/account/${addr}`,
  },
} as const;

export type ChainInfoLike = typeof BABYLON;


