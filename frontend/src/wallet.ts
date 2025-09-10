import type { Keplr } from "@keplr-wallet/types";
import { BABYLON } from "./chain";
import { SigningCosmWasmClient } from "@cosmjs/cosmwasm-stargate";
import { GasPrice } from "@cosmjs/stargate";

declare global {
  interface Window {
    keplr?: Keplr;
    getOfflineSigner?: (chainId: string) => any;
  }
}

export async function ensureChainSuggested(): Promise<void> {
  if (!window.keplr) throw new Error("Keplr not found. Install Keplr extension.");
  const experimentalSuggest = (window.keplr as any).experimentalSuggestChain;
  if (!experimentalSuggest) return; // Many chains are pre-added
  await experimentalSuggest({
    chainId: BABYLON.chainId,
    chainName: BABYLON.chainName,
    rpc: BABYLON.rpc,
    rest: BABYLON.rest,
    bip44: BABYLON.bip44,
    bech32Config: BABYLON.bech32Config,
    stakeCurrency: BABYLON.stakeCurrency,
    currencies: BABYLON.currencies,
    feeCurrencies: BABYLON.feeCurrencies,
    features: BABYLON.features as any,
  });
}

export async function connectKeplr() {
  if (!window.keplr) throw new Error("Keplr not found. Install Keplr extension.");
  await ensureChainSuggested();
  await window.keplr!.enable(BABYLON.chainId);
  const offlineSigner = window.getOfflineSigner!(BABYLON.chainId);
  const accounts = await offlineSigner.getAccounts();
  const address = accounts[0]?.address ?? "";
  const client = await SigningCosmWasmClient.connectWithSigner(
    BABYLON.rpc,
    offlineSigner,
    {
      gasPrice: GasPrice.fromString("0.025ubbn"),
    }
  );
  return { address, client };
}


