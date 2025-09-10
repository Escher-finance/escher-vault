import { useEffect, useState } from 'react'
import type { SigningCosmWasmClient } from '@cosmjs/cosmwasm-stargate'

type Props = {
  client: SigningCosmWasmClient | null
  contractAddress?: string
  user?: string
}

export function Redemptions({ client, contractAddress, user }: Props) {
  const [contract, setContract] = useState(contractAddress || '')
  const [target, setTarget] = useState(user || '')
  const [stats, setStats] = useState<any>(null)
  const [list, setList] = useState<any[]>([])
  const [error, setError] = useState('')
  const [loading, setLoading] = useState(false)
  const [txById, setTxById] = useState<Record<number, string>>({})
  // All requests pagination
  const [allList, setAllList] = useState<any[]>([])
  const [nextStartAfter, setNextStartAfter] = useState<number | null>(null)
  const [pageLimit, setPageLimit] = useState<number>(20)

  async function load() {
    if (!client || !contract) return
    setError(''); setLoading(true)
    try {
      // Token info canary (verifies connectivity/addr)
      try {
        await client.queryContractSmart(contract, { token_info: {} } as any)
      } catch (e:any) {
        setError(`token_info failed: ${e?.message || String(e)}`)
      }

      // Stats
      try {
        const s = await client.queryContractSmart(contract, { redemption_stats: {} } as any)
        setStats(s)
      } catch (e1:any) {
        // Retry with null variant form
        try {
          const s2 = await client.queryContractSmart(contract, { redemption_stats: null } as any)
          setStats(s2)
        } catch (e2:any) {
          setStats(null)
          setError(`stats: ${e2?.message || String(e2)}`)
        }
      }

      // User requests
      try {
        if (target) {
          const l = await client.queryContractSmart(contract, { user_redemption_requests: { user: target } } as any)
          setList(l.requests || [])
        } else {
          setList([])
        }
      } catch (e:any) {
        setList([])
        setError(`user_requests: ${e?.message || String(e)}`)
      }

      // All requests first page
      try {
        const ar = await client.queryContractSmart(contract, { all_redemption_requests: { start_after: null, limit: pageLimit } } as any)
        setAllList(ar.requests || [])
        setNextStartAfter(ar.next_start_after ?? null)
      } catch (e:any) {
        setAllList([])
        setNextStartAfter(null)
        setError(`all_requests: ${e?.message || String(e)}`)
      }
    } finally { setLoading(false) }
  }

  useEffect(()=>{ load() }, [client, contract, target])

  if (contractAddress && contract !== contractAddress) setContract(contractAddress)
  if (user && target !== user) setTarget(user)

  return (
    <div className="card" style={{ display: 'flex', flexDirection: 'column', gap: 12 }}>
      <h3>Redemption Requests</h3>
      <div style={{ display: 'flex', gap: 8, flexWrap: 'wrap' }}>
        <input placeholder="Contract (bbn1...)" value={contract} onChange={(e)=>setContract(e.target.value)} style={{ padding: 8 }} />
        <input placeholder="User (bbn1...)" value={target} onChange={(e)=>setTarget(e.target.value)} style={{ padding: 8 }} />
        <button onClick={load} disabled={loading}>{loading ? 'Loading...' : 'Refresh'}</button>
      </div>
      {error && <div style={{ color: 'red' }}>{error}</div>}

      <section>
        <h4>Stats</h4>
        <pre style={{ background: '#f3f4f6', padding: 12, borderRadius: 8, overflow: 'auto', border: '1px solid #e5e7eb', color: '#111' }}>{stats ? JSON.stringify(stats, null, 2) : 'n/a'}</pre>
      </section>

      <section>
        <h4>All Requests</h4>
        <div style={{ display: 'flex', gap: 8, alignItems: 'center' }}>
          <label>Page size
            <input type="number" min={1} max={200} value={pageLimit} onChange={(e)=>setPageLimit(Math.min(200, Math.max(1, Number(e.target.value)||20)))} style={{ marginLeft: 8, width: 80 }} />
          </label>
          <button disabled={loading} onClick={load}>Reload</button>
          <button disabled={loading || nextStartAfter===null} onClick={async ()=>{
            if (!client || !contract) return
            setError(''); setLoading(true)
            try {
              const ar = await client.queryContractSmart(contract, { all_redemption_requests: { start_after: nextStartAfter, limit: pageLimit } } as any)
              setAllList(prev => [...prev, ...(ar.requests || [])])
              setNextStartAfter(ar.next_start_after ?? null)
            } catch (e:any) { setError(e?.message||String(e)) } finally { setLoading(false) }
          }}>Load more</button>
        </div>
        <ul style={{ listStyle: 'none', padding: 0, margin: 0 }}>
          {allList.map((r)=> (
            <li key={`all-${r.id}`} style={{ borderBottom: '1px solid #e5e7eb', padding: '6px 0' }}>
              <div><strong>ID:</strong> {r.id}</div>
              <div><strong>Status:</strong> {typeof r.status === 'string' ? r.status : JSON.stringify(r.status)}</div>
              <div><strong>Owner:</strong> {r.owner}</div>
              <div><strong>Receiver:</strong> {r.receiver}</div>
              <div><strong>Shares locked:</strong> {r.shares_locked}</div>
              { (typeof r.status === 'string' ? r.status.toLowerCase() === 'pending' : true) && (
                <div style={{ display: 'flex', gap: 8, marginTop: 6, flexWrap: 'wrap' }}>
                  <input
                    placeholder="tx_hash"
                    value={txById[r.id] || ''}
                    onChange={(e)=> setTxById(prev => ({ ...prev, [r.id]: e.target.value }))}
                    style={{ padding: 8, minWidth: 280 }}
                  />
                  <button disabled={!client} onClick={async ()=>{
                    if (!client) return;
                    setError(''); setLoading(true);
                    try {
                      const msg = { complete_redemption: { redemption_id: Number(r.id), tx_hash: (txById[r.id]||'').trim() } } as any
                      // execute as current connected wallet
                      const res = await client.execute(user || '', contract, msg, 'auto')
                      console.log('complete_redemption tx', res.transactionHash)
                      await load()
                    } catch(e:any) {
                      setError(e?.message || String(e))
                    } finally { setLoading(false) }
                  }}>Complete</button>
                </div>
              )}
              {r.completed_at && <div><strong>Completed at:</strong> {r.completed_at}</div>}
              {r.completion_tx_hash && <div><strong>Tx:</strong> {r.completion_tx_hash}</div>}
            </li>
          ))}
          {!allList.length && <li>No requests</li>}
        </ul>
      </section>

      <section>
        <h4>User Requests</h4>
        <ul style={{ listStyle: 'none', padding: 0, margin: 0 }}>
          {list.map((r)=> (
            <li key={r.id} style={{ borderBottom: '1px solid #333', padding: '6px 0' }}>
              <div><strong>ID:</strong> {r.id}</div>
              <div><strong>Status:</strong> {typeof r.status === 'string' ? r.status : JSON.stringify(r.status)}</div>
              <div><strong>Owner:</strong> {r.owner}</div>
              <div><strong>Receiver:</strong> {r.receiver}</div>
              <div><strong>Shares locked:</strong> {r.shares_locked}</div>
              {r.completed_at && <div><strong>Completed at:</strong> {r.completed_at}</div>}
              {r.completion_tx_hash && <div><strong>Tx:</strong> {r.completion_tx_hash}</div>}
            </li>
          ))}
          {!list.length && <li>No user requests</li>}
        </ul>
      </section>
    </div>
  )
}


