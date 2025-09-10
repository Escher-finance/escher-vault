import { useEffect, useMemo, useState } from 'react'
import { BABYLON } from '../chain'

type Tx = {
  txhash: string
  height: string
  timestamp?: string
  code?: number
}

async function fetchTxs(contract: string, limit = 10): Promise<Tx[]> {
  const base = BABYLON.rest.replace(/\/$/, '')
  // Try both event keys commonly indexed by Cosmos SDK chains
  const urls = [
    `${base}/cosmos/tx/v1beta1/txs?events=wasm._contract_address='${contract}'&order_by=2&pagination.limit=${limit}`,
    `${base}/cosmos/tx/v1beta1/txs?events=wasm.contract_address='${contract}'&order_by=2&pagination.limit=${limit}`,
  ]
  for (const url of urls) {
    try {
      const res = await fetch(url)
      if (!res.ok) continue
      const data = await res.json()
      const txs = (data.tx_responses || []).map((t: any) => ({
        txhash: t.txhash,
        height: String(t.height),
        timestamp: t.timestamp,
        code: t.code,
      }))
      if (txs.length) return txs
    } catch (_) {
      // try next
    }
  }
  return []
}

type Props = { contractAddress?: string }

export function RecentTxs({ contractAddress }: Props) {
  const [contract, setContract] = useState(contractAddress || '')
  const [txs, setTxs] = useState<Tx[]>([])
  const [error, setError] = useState('')
  const [loading, setLoading] = useState(false)

  const valid = useMemo(()=>/^(bbn1)[0-9a-z]{38,}$/.test(contract), [contract])

  async function load() {
    if (!valid) return
    setLoading(true); setError('')
    try {
      const list = await fetchTxs(contract, 15)
      setTxs(list)
    } catch (e:any) { setError(e?.message||String(e)) } finally { setLoading(false) }
  }

  useEffect(()=>{ if (contract) load() }, [contract])

  if (contractAddress && contract !== contractAddress) {
    setContract(contractAddress)
  }

  return (
    <div className="card" style={{ display: 'flex', flexDirection: 'column', gap: 12 }}>
      <h3>Recent Transactions</h3>
      <input placeholder="Contract address (bbn1...)" value={contract} onChange={(e)=>setContract(e.target.value)} style={{ padding: 8 }} />
      <button onClick={load} disabled={!valid || loading}>{loading ? 'Loading...' : 'Refresh'}</button>
      {error && <div style={{ color: 'red' }}>{error}</div>}
      <ul style={{ listStyle: 'none', padding: 0, margin: 0 }}>
        {txs.map((t)=> (
          <li key={t.txhash} style={{ display: 'flex', justifyContent: 'space-between', gap: 8, borderBottom: '1px solid #e5e7eb', padding: '8px 0' }}>
            <a href={BABYLON.explorer.tx(t.txhash)} target="_blank" rel="noreferrer" style={{ fontFamily: 'monospace' }}>{t.txhash.slice(0,10)}…</a>
            <span>h#{t.height}</span>
            <span>{t.timestamp?.replace('T',' ').replace('Z','') || ''}</span>
            <span style={{ color: t.code && t.code !== 0 ? '#b91c1c' : '#065f46' }}>{t.code && t.code !== 0 ? 'failed' : 'success'}</span>
          </li>
        ))}
        {!txs.length && !loading && <li>No transactions found</li>}
      </ul>
    </div>
  )
}


