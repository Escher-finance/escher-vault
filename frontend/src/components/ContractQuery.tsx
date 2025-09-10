import { useState } from 'react'
import type { SigningCosmWasmClient } from '@cosmjs/cosmwasm-stargate'

type Props = {
  client?: SigningCosmWasmClient | null
  contractAddress?: string
}

export function ContractQuery({ client, contractAddress }: Props) {
  const [contract, setContract] = useState("")
  const [msg, setMsg] = useState("{\n  \"token_info\": {}\n}")
  const [result, setResult] = useState<string>("")
  const [error, setError] = useState<string>("")
  const [loading, setLoading] = useState(false)

  // keep local input in sync if parent provides a value
  if (contractAddress && contract !== contractAddress) {
    // lightweight sync without useEffect to avoid extra import
    setContract(contractAddress)
  }

  return (
    <div className="card" style={{ display: 'flex', flexDirection: 'column', gap: 8 }}>
      <h3>Contract Query</h3>
      <input
        placeholder="Contract address (bbn1...)"
        value={contract}
        onChange={(e) => setContract(e.target.value)}
        style={{ padding: 8 }}
      />
      <textarea
        rows={6}
        value={msg}
        onChange={(e) => setMsg(e.target.value)}
        style={{ padding: 8, fontFamily: 'monospace' }}
      />
      <button disabled={!client || loading} onClick={async () => {
        setError(""); setResult(""); setLoading(true);
        try {
          if (!client) throw new Error('Connect wallet first');
          const parsed = JSON.parse(msg);
          const res = await client.queryContractSmart(contract, parsed);
          setResult(JSON.stringify(res, null, 2));
        } catch (e: any) {
          setError(e?.message || String(e));
        } finally {
          setLoading(false);
        }
      }}>
        {loading ? 'Querying...' : 'Query'}
      </button>
      {error && <div style={{ color: 'red' }}>{error}</div>}
      {result && (
        <pre style={{ background: '#111', padding: 12, borderRadius: 8, overflow: 'auto' }}>{result}</pre>
      )}
    </div>
  )
}


