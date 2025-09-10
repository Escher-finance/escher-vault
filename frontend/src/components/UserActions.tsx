import { useEffect, useState } from 'react'
import type { SigningCosmWasmClient } from '@cosmjs/cosmwasm-stargate'
import { coin } from '@cosmjs/proto-signing'
import { BABYLON } from '../chain'

type Props = {
  client: SigningCosmWasmClient | null
  sender: string
  contractAddress?: string
  userShareMin?: string
  shareDecimals?: number
}

async function doExec(
  client: SigningCosmWasmClient,
  sender: string,
  contract: string,
  msg: Record<string, unknown>
) {
  return client.execute(sender, contract, msg, 'auto')
}

export function UserActions({ client, sender, contractAddress, userShareMin = '0', shareDecimals = 6 }: Props) {
  const [contract, setContract] = useState(contractAddress || '')
  const [status, setStatus] = useState('')
  const [error, setError] = useState('')

  const [depositAssets, setDepositAssets] = useState('0')

  const [redeemShares, setRedeemShares] = useState('0')
  const [redeemReceiver, setRedeemReceiver] = useState('')
  const [redeemOwner, setRedeemOwner] = useState('')

  function ensure() {
    if (!client) throw new Error('Connect wallet first')
    if (!contract) throw new Error('Enter contract address')
    return client
  }

  // Default receiver/owner to connected wallet
  useEffect(() => {
    if (sender) {
      if (!redeemReceiver) setRedeemReceiver(sender)
      if (!redeemOwner) setRedeemOwner(sender)
    }
  // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [sender])

  // Convert human units (BBN, shares) -> minimal (uBBN, min-share)
  function toMinimalUnits(input: string, decimals = 6): string {
    const s = (input || '').trim()
    if (!s) return '0'
    const [whole, frac = ''] = s.split('.')
    const cleanedWhole = whole.replace(/\D/g, '') || '0'
    const cleanedFrac = frac.replace(/\D/g, '')
    const fracPadded = (cleanedFrac + '0'.repeat(decimals)).slice(0, decimals)
    // remove leading zeros
    const result = `${cleanedWhole}${fracPadded}`.replace(/^0+/, '')
    return result === '' ? '0' : result
  }

  function toHuman(min: string, decimals = 6): string {
    const n = Number(min || '0')
    if (!isFinite(n)) return '0'
    return (n / Math.pow(10, decimals)).toFixed(decimals)
  }

  if (contractAddress && contract !== contractAddress) {
    setContract(contractAddress)
  }

  return (
    <div className="card" style={{ display: 'flex', flexDirection: 'column', gap: 16 }}>
      <h3>User Actions</h3>
      <input
        placeholder="Contract address (bbn1...)"
        value={contract}
        onChange={(e) => setContract(e.target.value)}
        style={{ padding: 8 }}
      />

      <section>
        <h4>Deposit</h4>
        <div style={{ display: 'flex', gap: 8, alignItems: 'center' }}>
          <input placeholder="assets (BBN, e.g. 1.25)" value={depositAssets} onChange={(e)=>setDepositAssets(e.target.value)} />
          <span style={{ fontSize: 12, color: '#6b7280' }}>receiver: {sender || '(connect wallet)'}</span>
          <button onClick={async ()=>{
            setStatus(''); setError('');
            try {
              const c = ensure();
              const amt = toMinimalUnits(depositAssets)
              const msg = { deposit: { assets: amt, receiver: sender } } as any
              const funds = [coin(amt, BABYLON.stakeCurrency.coinMinimalDenom)]
              const res = await c.execute(sender, contract, msg, 'auto', undefined, funds)
              setStatus(`OK: ${res.transactionHash}`)
            } catch (e:any) { setError(e?.message||String(e)) }
          }}>Deposit</button>
        </div>
      </section>

      <section>
        <h4>Request Redeem</h4>
        <div style={{ display: 'flex', gap: 8, flexWrap: 'wrap' }}>
          <input placeholder="shares (e.g. 1.25)" value={redeemShares} onChange={(e)=>setRedeemShares(e.target.value)} />
          <button type="button" onClick={()=>setRedeemShares(toHuman(userShareMin, shareDecimals))}>Max</button>
          <input placeholder="receiver (bbn1...)" value={redeemReceiver} onChange={(e)=>setRedeemReceiver(e.target.value)} />
          <input placeholder="owner (bbn1...)" value={redeemOwner} onChange={(e)=>setRedeemOwner(e.target.value)} />
          <button onClick={async ()=>{
            setStatus(''); setError('');
            try {
              const c = ensure();
              const amt = toMinimalUnits(redeemShares)
              const msg = { request_redeem: { shares: amt, receiver: redeemReceiver || sender, owner: redeemOwner || sender } } as any
              const res = await doExec(c, sender, contract, msg)
              setStatus(`OK: ${res.transactionHash}`)
            } catch (e:any) { setError(e?.message||String(e)) }
          }}>Request</button>
        </div>
      </section>

      {status && <div style={{ color: 'lime' }}>{status}</div>}
      {error && <div style={{ color: 'red' }}>{error}</div>}
    </div>
  )
}


