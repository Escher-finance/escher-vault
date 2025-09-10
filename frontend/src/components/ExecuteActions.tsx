import { useState } from 'react'
import type { SigningCosmWasmClient } from '@cosmjs/cosmwasm-stargate'

type Props = {
  client: SigningCosmWasmClient | null
  sender: string
  contractAddress?: string
}

async function doExec(
  client: SigningCosmWasmClient,
  sender: string,
  contract: string,
  msg: Record<string, unknown>
) {
  return client.execute(sender, contract, msg, 'auto')
}

export function ExecuteActions({ client, sender, contractAddress }: Props) {
  const [contract, setContract] = useState(contractAddress || '')
  const [status, setStatus] = useState('')
  const [error, setError] = useState('')

  const [depositAssets, setDepositAssets] = useState('0')
  const [depositReceiver, setDepositReceiver] = useState('')

  const [bondAmount, setBondAmount] = useState('0')
  const [bondSalt, setBondSalt] = useState('')
  const [bondSlippage, setBondSlippage] = useState('')

  const [unbondAmount, setUnbondAmount] = useState('0')

  const [addLiqUnderlying, setAddLiqUnderlying] = useState('0')
  const [removeLiqLp, setRemoveLiqLp] = useState('0')

  const [swapAmount, setSwapAmount] = useState('0')
  const [swapAssetInfo, setSwapAssetInfo] = useState('{"native_token":{"denom":"ubbn"}}')

  const [redeemShares, setRedeemShares] = useState('0')
  const [redeemReceiver, setRedeemReceiver] = useState('')
  const [redeemOwner, setRedeemOwner] = useState('')

  const [completeId, setCompleteId] = useState('0')
  const [completeTxHash, setCompleteTxHash] = useState('')

  function ensure() {
    if (!client) throw new Error('Connect wallet first')
    if (!contract) throw new Error('Enter contract address')
    return client
  }

  if (contractAddress && contract !== contractAddress) {
    setContract(contractAddress)
  }

  return (
    <div className="card" style={{ display: 'flex', flexDirection: 'column', gap: 16 }}>
      <h3>Execute Actions</h3>
      <input
        placeholder="Contract address (bbn1...)"
        value={contract}
        onChange={(e) => setContract(e.target.value)}
        style={{ padding: 8 }}
      />

      <section>
        <h4>Deposit</h4>
        <div style={{ display: 'flex', gap: 8 }}>
          <input placeholder="assets (Uint128)" value={depositAssets} onChange={(e)=>setDepositAssets(e.target.value)} />
          <input placeholder="receiver (bbn1...)" value={depositReceiver} onChange={(e)=>setDepositReceiver(e.target.value)} />
          <button onClick={async ()=>{
            setStatus(''); setError('');
            try {
              const c = ensure();
              const msg = { deposit: { assets: depositAssets, receiver: depositReceiver || sender } } as any
              const res = await doExec(c, sender, contract, msg)
              setStatus(`OK: ${res.transactionHash}`)
            } catch (e:any) { setError(e?.message||String(e)) }
          }}>Deposit</button>
        </div>
      </section>

      <section>
        <h4>Bond</h4>
        <div style={{ display: 'flex', gap: 8, flexWrap: 'wrap' }}>
          <input placeholder="amount" value={bondAmount} onChange={(e)=>setBondAmount(e.target.value)} />
          <input placeholder="salt" value={bondSalt} onChange={(e)=>setBondSalt(e.target.value)} />
          <input placeholder="slippage (optional, e.g. 0.01)" value={bondSlippage} onChange={(e)=>setBondSlippage(e.target.value)} />
          <button onClick={async ()=>{
            setStatus(''); setError('');
            try {
              const c = ensure();
              const sl = bondSlippage ? bondSlippage : undefined
              const msg = { bond: { amount: bondAmount, salt: bondSalt, slippage: sl } } as any
              const res = await doExec(c, sender, contract, msg)
              setStatus(`OK: ${res.transactionHash}`)
            } catch (e:any) { setError(e?.message||String(e)) }
          }}>Bond</button>
        </div>
      </section>

      <section>
        <h4>Unbond</h4>
        <div style={{ display: 'flex', gap: 8 }}>
          <input placeholder="amount" value={unbondAmount} onChange={(e)=>setUnbondAmount(e.target.value)} />
          <button onClick={async ()=>{
            setStatus(''); setError('');
            try {
              const c = ensure();
              const msg = { unbond: { amount: unbondAmount } } as any
              const res = await doExec(c, sender, contract, msg)
              setStatus(`OK: ${res.transactionHash}`)
            } catch (e:any) { setError(e?.message||String(e)) }
          }}>Unbond</button>
        </div>
      </section>

      <section>
        <h4>Add Liquidity</h4>
        <div style={{ display: 'flex', gap: 8 }}>
          <input placeholder="underlying_token_amount" value={addLiqUnderlying} onChange={(e)=>setAddLiqUnderlying(e.target.value)} />
          <button onClick={async ()=>{
            setStatus(''); setError('');
            try {
              const c = ensure();
              const msg = { add_liquidity: { underlying_token_amount: addLiqUnderlying } } as any
              const res = await doExec(c, sender, contract, msg)
              setStatus(`OK: ${res.transactionHash}`)
            } catch (e:any) { setError(e?.message||String(e)) }
          }}>Add</button>
        </div>
      </section>

      <section>
        <h4>Remove Liquidity</h4>
        <div style={{ display: 'flex', gap: 8 }}>
          <input placeholder="lp_token_amount" value={removeLiqLp} onChange={(e)=>setRemoveLiqLp(e.target.value)} />
          <button onClick={async ()=>{
            setStatus(''); setError('');
            try {
              const c = ensure();
              const msg = { remove_liquidity: { lp_token_amount: removeLiqLp } } as any
              const res = await doExec(c, sender, contract, msg)
              setStatus(`OK: ${res.transactionHash}`)
            } catch (e:any) { setError(e?.message||String(e)) }
          }}>Remove</button>
        </div>
      </section>

      <section>
        <h4>Claim Incentives</h4>
        <button onClick={async ()=>{
          setStatus(''); setError('');
          try {
            const c = ensure();
            const msg = { claim_incentives: {} } as any
            const res = await doExec(c, sender, contract, msg)
            setStatus(`OK: ${res.transactionHash}`)
          } catch (e:any) { setError(e?.message||String(e)) }
        }}>Claim</button>
      </section>

      <section>
        <h4>Swap</h4>
        <div style={{ display: 'flex', gap: 8, flexWrap: 'wrap' }}>
          <input placeholder="amount" value={swapAmount} onChange={(e)=>setSwapAmount(e.target.value)} />
          <input placeholder='asset_info JSON' value={swapAssetInfo} onChange={(e)=>setSwapAssetInfo(e.target.value)} style={{ minWidth: 360 }} />
          <button onClick={async ()=>{
            setStatus(''); setError('');
            try {
              const c = ensure();
              const assetInfo = JSON.parse(swapAssetInfo)
              const msg = { swap: { amount: swapAmount, asset_info: assetInfo } } as any
              const res = await doExec(c, sender, contract, msg)
              setStatus(`OK: ${res.transactionHash}`)
            } catch (e:any) { setError(e?.message||String(e)) }
          }}>Swap</button>
        </div>
      </section>

      <section>
        <h4>Request Redeem</h4>
        <div style={{ display: 'flex', gap: 8, flexWrap: 'wrap' }}>
          <input placeholder="shares" value={redeemShares} onChange={(e)=>setRedeemShares(e.target.value)} />
          <input placeholder="receiver (bbn1...)" value={redeemReceiver} onChange={(e)=>setRedeemReceiver(e.target.value)} />
          <input placeholder="owner (bbn1...)" value={redeemOwner} onChange={(e)=>setRedeemOwner(e.target.value)} />
          <button onClick={async ()=>{
            setStatus(''); setError('');
            try {
              const c = ensure();
              const msg = { request_redeem: { shares: redeemShares, receiver: redeemReceiver || sender, owner: redeemOwner || sender } } as any
              const res = await doExec(c, sender, contract, msg)
              setStatus(`OK: ${res.transactionHash}`)
            } catch (e:any) { setError(e?.message||String(e)) }
          }}>Request</button>
        </div>
      </section>

      <section>
        <h4>Complete Redemption</h4>
        <div style={{ display: 'flex', gap: 8 }}>
          <input placeholder="redemption_id (u64)" value={completeId} onChange={(e)=>setCompleteId(e.target.value)} />
          <input placeholder="tx_hash" value={completeTxHash} onChange={(e)=>setCompleteTxHash(e.target.value)} />
          <button onClick={async ()=>{
            setStatus(''); setError('');
            try {
              const c = ensure();
              const msg = { complete_redemption: { redemption_id: Number(completeId), tx_hash: completeTxHash } } as any
              const res = await doExec(c, sender, contract, msg)
              setStatus(`OK: ${res.transactionHash}`)
            } catch (e:any) { setError(e?.message||String(e)) }
          }}>Complete</button>
        </div>
      </section>

      {status && <div style={{ color: 'lime' }}>{status}</div>}
      {error && <div style={{ color: 'red' }}>{error}</div>}
    </div>
  )
}


