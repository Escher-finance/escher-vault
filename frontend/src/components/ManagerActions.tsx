import { useEffect, useState } from 'react'
import type { SigningCosmWasmClient } from '@cosmjs/cosmwasm-stargate'
import { BABYLON } from '../chain'

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

export function ManagerActions({ client, sender, contractAddress }: Props) {
  const [contract, setContract] = useState(contractAddress || '')
  const [status, setStatus] = useState('')
  const [error, setError] = useState('')

  const [bondAmount, setBondAmount] = useState('0')
  const [bondSalt, setBondSalt] = useState('')
  const [bondSlippage, setBondSlippage] = useState('')

  const [addLiqUnderlying, setAddLiqUnderlying] = useState('0')
  const [removeLiqLp, setRemoveLiqLp] = useState('0')

  const [swapAmount, setSwapAmount] = useState('0')
  const [swapAssetInfo, setSwapAssetInfo] = useState('{"native_token":{"denom":"ubbn"}}')

  const [completeId, setCompleteId] = useState('0')
  const [completeTxHash, setCompleteTxHash] = useState('')

  function ensure() {
    if (!client) throw new Error('Connect wallet first')
    if (!contract) throw new Error('Enter contract address')
    return client
  }

  function toMinimalUnits(input: string): string {
    const s = (input || '').trim()
    if (!s) return '0'
    if (s.includes('.')) {
      const v = Number(s)
      if (Number.isNaN(v)) return '0'
      return Math.round(v * 1_000_000).toString()
    }
    return s
  }

  if (contractAddress && contract !== contractAddress) {
    setContract(contractAddress)
  }

  function randomSalt(bytes = 32): string {
    try {
      const arr = new Uint8Array(bytes)
      crypto.getRandomValues(arr)
      const hex = Array.from(arr).map(b=>b.toString(16).padStart(2,'0')).join('')
      return `0x${hex}`
    } catch {
      // Fallback: non-crypto random (still 32 bytes, hex, prefixed)
      let hex = ''
      for (let i=0;i<bytes;i++) hex += Math.floor(Math.random()*256).toString(16).padStart(2,'0')
      return `0x${hex}`
    }
  }

  // Initialize a salt if empty
  useEffect(()=>{
    if (!bondSalt) setBondSalt(randomSalt())
  // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [contract])

  // Vault positions and config
  const [lpTokenAmount, setLpTokenAmount] = useState<string>('')
  const [towerConfig, setTowerConfig] = useState<any>(null)
  const [vaultUbbn, setVaultUbbn] = useState<string>('0')
  const [pendingIncentives, setPendingIncentives] = useState<any[]>([])

  async function refreshPositions() {
    setError('')
    try {
      if (!client || !contract) return
      const cfg = await client.queryContractSmart(contract, { config: {} } as any)
      setTowerConfig(cfg?.tower_config)
      const lp = await client.queryContractSmart(contract, { lp_position: {} } as any)
      setLpTokenAmount(lp?.lp_token_amount || '0')
      const bal = await client.getBalance(contract, BABYLON.stakeCurrency.coinMinimalDenom)
      setVaultUbbn(bal.amount || '0')
      try {
        const inc = await client.queryContractSmart(contract, { all_pending_incentives: {} } as any)
        setPendingIncentives(inc?.incentives || [])
      } catch (_) {
        setPendingIncentives([])
      }
    } catch (e:any) {
      setError(e?.message || String(e))
    }
  }

  useEffect(()=>{ refreshPositions() }, [client, contract])

  return (
    <div className="card" style={{ display: 'flex', flexDirection: 'column', gap: 16 }}>
      <h3>Manager Actions</h3>
      <input
        placeholder="Contract address (bbn1...)"
        value={contract}
        onChange={(e) => setContract(e.target.value)}
        style={{ padding: 8 }}
      />

      <section>
        <h4>Bond</h4>
        <div style={{ display: 'flex', gap: 8, flexWrap: 'wrap' }}>
          <input placeholder="amount" value={bondAmount} onChange={(e)=>setBondAmount(e.target.value)} />
          <input placeholder="salt" value={bondSalt} onChange={(e)=>setBondSalt(e.target.value)} />
          <button type="button" onClick={()=>setBondSalt(randomSalt())}>Regenerate salt</button>
          <input placeholder="slippage (optional)" value={bondSlippage} onChange={(e)=>setBondSlippage(e.target.value)} />
          <div style={{ fontSize: 12, color: '#6b7280' }}>salt must be 0x + 64 hex chars</div>
          <div style={{ fontSize: 12, color: '#374151' }}>vault balance: {vaultUbbn} ubbn (~{(Number(vaultUbbn)/1_000_000).toFixed(6)} BBN)</div>
          <button type="button" onClick={()=>{
            const half = Math.floor(Number(vaultUbbn)/2)
            setBondAmount((half/1_000_000).toString())
          }}>Set 50%</button>
          <button type="button" onClick={async ()=>{
            setStatus(''); setError('');
            try {
              const c = ensure();
              const half = Math.floor(Number(vaultUbbn)/2)
              const salt = bondSalt || randomSalt()
              const sl = bondSlippage ? bondSlippage : undefined
              const msg = { bond: { amount: String(half), salt, slippage: sl } } as any
              const res = await doExec(c, sender, contract, msg)
              setStatus(`OK: ${res.transactionHash}`)
              await refreshPositions()
            } catch (e:any) { setError(e?.message||String(e)) }
          }}>Bond 50%</button>
          <button onClick={async ()=>{
            setStatus(''); setError('');
            try {
              const c = ensure();
              const sl = bondSlippage ? bondSlippage : undefined
              const msg = { bond: { amount: toMinimalUnits(bondAmount), salt: bondSalt, slippage: sl } } as any
              const res = await doExec(c, sender, contract, msg)
              setStatus(`OK: ${res.transactionHash}`)
              await refreshPositions()
            } catch (e:any) { setError(e?.message||String(e)) }
          }}>Bond</button>
        </div>
      </section>

      <section>
        <h4>Provide Liquidity</h4>
        <div style={{ display: 'flex', gap: 8 }}>
          <input placeholder="underlying_token_amount" value={addLiqUnderlying} onChange={(e)=>setAddLiqUnderlying(e.target.value)} />
          <button onClick={async ()=>{
            setStatus(''); setError('');
            try {
              const c = ensure();
              const msg = { add_liquidity: { underlying_token_amount: toMinimalUnits(addLiqUnderlying) } } as any
              const res = await doExec(c, sender, contract, msg)
              setStatus(`OK: ${res.transactionHash}`)
              await refreshPositions()
            } catch (e:any) { setError(e?.message||String(e)) }
          }}>Provide</button>
        </div>
      </section>

      <section>
        <h4>Withdraw Liquidity</h4>
        <div style={{ display: 'flex', gap: 8 }}>
          <input placeholder="lp_token_amount" value={removeLiqLp} onChange={(e)=>setRemoveLiqLp(e.target.value)} />
          <button type="button" onClick={()=> setRemoveLiqLp((Number(lpTokenAmount)/1_000_000).toString())}>Max</button>
          <button onClick={async ()=>{
            setStatus(''); setError('');
            try {
              const c = ensure();
              const msg = { remove_liquidity: { lp_token_amount: toMinimalUnits(removeLiqLp) } } as any
              const res = await doExec(c, sender, contract, msg)
              setStatus(`OK: ${res.transactionHash}`)
              await refreshPositions()
            } catch (e:any) { setError(e?.message||String(e)) }
          }}>Withdraw</button>
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
              const msg = { swap: { amount: toMinimalUnits(swapAmount), asset_info: assetInfo } } as any
              const res = await doExec(c, sender, contract, msg)
              setStatus(`OK: ${res.transactionHash}`)
            } catch (e:any) { setError(e?.message||String(e)) }
          }}>Swap</button>
        </div>
      </section>

      <section>
        <h4>Complete Redemption</h4>
        <div style={{ display: 'flex', gap: 8, flexWrap: 'wrap' }}>
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

      <section>
        <h4>Vault Positions</h4>
        <button type="button" onClick={refreshPositions}>Refresh positions</button>
        <div style={{ marginTop: 8 }}>
          <div><strong>LP token amount</strong>: {lpTokenAmount || '0'}</div>
          {pendingIncentives && pendingIncentives.length > 0 && (
            <div style={{ marginTop: 8 }}>
              <div><strong>Pending incentives</strong>:</div>
              <ul>
                {pendingIncentives.map((a, idx)=> (
                  <li key={idx}>{JSON.stringify(a)}</li>
                ))}
              </ul>
            </div>
          )}
          {towerConfig && (
            <div style={{ marginTop: 8 }}>
              <div><strong>LP pool</strong>: {towerConfig.lp}</div>
              <div><strong>LP token</strong>: {towerConfig.lp_token}</div>
              <div><strong>Underlying asset</strong>: {JSON.stringify(towerConfig.lp_underlying_asset)}</div>
              <div><strong>Other asset</strong>: {JSON.stringify(towerConfig.lp_other_asset)}</div>
              <div><strong>Incentives</strong>: {JSON.stringify(towerConfig.lp_incentives)}</div>
            </div>
          )}
        </div>
      </section>
    </div>
  )
}


