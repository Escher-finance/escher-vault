import { useState } from 'react'
import './App.css'
import { connectKeplr } from './wallet'
import { BABYLON } from './chain'
import type { SigningCosmWasmClient } from '@cosmjs/cosmwasm-stargate'
import { ContractQuery } from './components/ContractQuery'
import { ExecuteActions } from './components/ExecuteActions'
import { UserActions } from './components/UserActions'
import { ManagerActions } from './components/ManagerActions'
import { OracleActions } from './components/OracleActions'
import { RecentTxs } from './components/RecentTxs'
import { Redemptions } from './components/Redemptions'

function App() {
  const [address, setAddress] = useState<string>("")
  const [balance, setBalance] = useState<string>("")
  const [error, setError] = useState<string>("")
  const [connecting, setConnecting] = useState(false)
  const [client, setClient] = useState<SigningCosmWasmClient | null>(null)
  const [contract, setContract] = useState<string>("bbn1mzgguj0382s870h2fgnsu7nu658nmrduuvydz354cmzhy3czwafq44ckt3")
  const [tab, setTab] = useState<'user'|'manager'|'oracle'|'activity'|'dev'>('user')
  const [exchangeRate, setExchangeRate] = useState<string>("")
  const [myShares, setMyShares] = useState<string>("")
  const [totalShares, setTotalShares] = useState<string>("")
  const [vaultUbbn, setVaultUbbn] = useState<string>("")
  const feeRecipient = "bbn1y3u4mw39adngenlzwqm6hz60flz25gsxh2s9x4"
  const [feeShares, setFeeShares] = useState<string>("")
  const [shareDecimals, setShareDecimals] = useState<number>(6)

  async function refreshStats(c?: SigningCosmWasmClient | null, ct?: string, addr?: string) {
    const cc = c ?? client
    const cn = ct ?? contract
    const ad = addr ?? address
    if (!cc || !cn) return
    try {
      const er = await cc.queryContractSmart(cn, { exchange_rate: {} } as any)
      setExchangeRate(er.exchange_rate)
    } catch {}
    try {
      if (ad) {
        const bal = await cc.queryContractSmart(cn, { balance: { address: ad } } as any)
        setMyShares(bal.balance)
      } else {
        setMyShares("")
      }
    } catch {}
    try {
      const fb = await cc.queryContractSmart(cn, { balance: { address: feeRecipient } } as any)
      setFeeShares(fb.balance)
    } catch {}
    try {
      const ti = await cc.queryContractSmart(cn, { token_info: {} } as any)
      setTotalShares(ti.total_supply)
      if (typeof ti.decimals === 'number') setShareDecimals(ti.decimals)
    } catch {}
    try {
      const bal = await cc.getBalance(cn, BABYLON.stakeCurrency.coinMinimalDenom)
      setVaultUbbn(bal.amount || "")
    } catch {}
  }

  function formatHuman(min: string | number | undefined, decimals = shareDecimals): string {
    const n = typeof min === 'string' ? Number(min) : (min ?? 0)
    if (!isFinite(n)) return '0'
    return (n / Math.pow(10, decimals)).toFixed(decimals)
  }

  return (
    <>
      <h1>CW4626 Vault UI (Babylon Testnet)</h1>
      <div className="card" style={{ display: 'flex', flexDirection: 'column', gap: 12 }}>
        <button disabled={connecting} onClick={async () => {
          setError("");
          setConnecting(true);
          try {
            const { address, client } = await connectKeplr();
            setAddress(address);
            setClient(client);
            const bal = await client.getBalance(address, BABYLON.stakeCurrency.coinMinimalDenom);
            setBalance(`${bal.amount} ${BABYLON.stakeCurrency.coinDenom}`);
            refreshStats(client, undefined, address)
          } catch (e: any) {
            setError(e?.message || String(e));
          } finally {
            setConnecting(false);
          }
        }}>
          {address ? 'Connected' : 'Connect Keplr'}
        </button>

        {address && (
          <div>
            <div><strong>Address:</strong> {address}</div>
            <div><strong>Balance:</strong> {balance || 'loading...'}</div>
          </div>
        )}
        {error && <div style={{ color: 'red' }}>{error}</div>}
      </div>
      <div className="card" style={{ display: 'flex', gap: 16, alignItems: 'center', flexWrap: 'wrap' }}>
        <div><strong>Exchange rate</strong>: {exchangeRate || 'n/a'}</div>
        <div><strong>My shares</strong>: {formatHuman(myShares)} ({myShares || '0'})</div>
        <div><strong>Total shares</strong>: {formatHuman(totalShares)} ({totalShares || 'n/a'})</div>
        <div><strong>Vault balance</strong>: {formatHuman(vaultUbbn)} BBN ({vaultUbbn || '0'} ubbn)</div>
        <div><strong>Fee recipient shares</strong>: {formatHuman(feeShares)} ({feeShares || '0'})</div>
      </div>
      <div className="card" style={{ display: 'flex', flexDirection: 'column', gap: 8 }}>
        <h3>Shared Contract Address</h3>
        <input
          placeholder="bbn1..."
          value={contract}
          onChange={(e) => setContract(e.target.value)}
          style={{ padding: 8 }}
        />
        <button onClick={()=>refreshStats()}>Refresh stats</button>
      </div>

      <div className="card" style={{ display: 'flex', gap: 8 }}>
        <button onClick={()=>setTab('user')} disabled={tab==='user'}>User</button>
        <button onClick={()=>setTab('manager')} disabled={tab==='manager'}>Manager</button>
        <button onClick={()=>setTab('oracle')} disabled={tab==='oracle'}>Oracle</button>
        <button onClick={()=>setTab('activity')} disabled={tab==='activity'}>Activity</button>
        <button onClick={()=>setTab('dev')} disabled={tab==='dev'}>Dev</button>
      </div>

      {tab==='user' && (
        <UserActions client={client} sender={address} contractAddress={contract} userShareMin={myShares} shareDecimals={shareDecimals} />
      )}
      {tab==='manager' && (
        <ManagerActions client={client} sender={address} contractAddress={contract} />
      )}
      {tab==='oracle' && (
        <OracleActions client={client} sender={address} contractAddress={contract} />
      )}
      {tab==='activity' && (
        <>
          <RecentTxs contractAddress={contract} />
          <Redemptions client={client} contractAddress={contract} user={address} />
        </>
      )}
      {tab==='dev' && (
        <>
          <ContractQuery client={client} contractAddress={contract} />
          <ExecuteActions client={client} sender={address} contractAddress={contract} />
        </>
      )}
    </>
  )
}

export default App
