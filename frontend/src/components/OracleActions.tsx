import { useEffect, useState } from 'react'
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

export function OracleActions({ client, sender, contractAddress }: Props) {
  const [contract, setContract] = useState(contractAddress || '')
  const [status, setStatus] = useState('')
  const [error, setError] = useState('')
  const [prices, setPrices] = useState('')
  const [tokens, setTokens] = useState<string[]>([])
  const [pricesView, setPricesView] = useState<string>('')
  const [oracleAddresses, setOracleAddresses] = useState<string[]>([])
  const [isOracle, setIsOracle] = useState<boolean>(false)

  function ensure() {
    if (!client) throw new Error('Connect wallet first')
    if (!contract) throw new Error('Enter contract address')
    return client
  }

  async function refresh() {
    setError('');
    try {
      if (!client || !contract) return
      const t = await client.queryContractSmart(contract, { oracle_tokens_list: {} })
      const p = await client.queryContractSmart(contract, { oracle_prices: {} })
      const r = await client.queryContractSmart(contract, { role: { kind: { oracle: {} } } } as any)
      setTokens(t.tokens || [])
      setPricesView(JSON.stringify(p.prices || {}, null, 2))
      const addrs: string[] = r.addresses || []
      setOracleAddresses(addrs)
      setIsOracle(!!sender && addrs.includes(sender))
    } catch (e:any) {
      setError(e?.message || String(e))
    }
  }

  useEffect(()=>{ refresh() }, [client, contract])

  if (contractAddress && contract !== contractAddress) {
    setContract(contractAddress)
  }

  return (
    <div className="card" style={{ display: 'flex', flexDirection: 'column', gap: 16 }}>
      <h3>Oracle Actions</h3>
      <input
        placeholder="Contract address (bbn1...)"
        value={contract}
        onChange={(e) => setContract(e.target.value)}
        style={{ padding: 8 }}
      />

      <div>
        <strong>Your address:</strong> {sender || '(not connected)'}
      </div>
      <div style={{ color: isOracle ? 'lime' : 'orange' }}>
        {isOracle ? 'You have Oracle role.' : 'You do not have Oracle role; update will fail.'}
      </div>

      <section>
        <h4>Set Prices</h4>
        <textarea rows={6} placeholder='{"token_addr":"1.23","token2":"0.5"}' value={prices} onChange={(e)=>setPrices(e.target.value)} style={{ fontFamily: 'monospace', padding: 8 }} />
        <button onClick={async ()=>{
          setStatus(''); setError('');
          try {
            const c = ensure();
            const parsed = prices ? JSON.parse(prices) : {}
            const msg = { oracle_update_prices: { prices: parsed } } as any
            const res = await doExec(c, sender, contract, msg)
            setStatus(`OK: ${res.transactionHash}`)
            await refresh()
          } catch (e:any) { setError(e?.message||String(e)) }
        }} disabled={!isOracle}>Update Prices</button>
      </section>

      <section>
        <h4>Oracle Tokens</h4>
        <ul>
          {tokens.map((t)=> <li key={t}>{t}</li>)}
        </ul>
      </section>

      <section>
        <h4>Oracle Role Addresses</h4>
        <ul>
          {oracleAddresses.map((a)=> <li key={a}>{a}</li>)}
          {!oracleAddresses.length && <li>none</li>}
        </ul>
      </section>

      <section>
        <h4>Oracle Prices</h4>
        <pre style={{ background: '#f3f4f6', padding: 12, borderRadius: 8, border: '1px solid #e5e7eb', color: '#111' }}>{pricesView}</pre>
      </section>

      {status && <div style={{ color: 'lime' }}>{status}</div>}
      {error && <div style={{ color: 'red' }}>{error}</div>}
    </div>
  )
}


