# 🚨 Asset Info Vulnerabilities Analysis

## Critical Vulnerabilities in `asset_info.rs`

### 1. 🔴 **CRITICAL: Hardcoded Native Token Decimals**

#### **The Problem:**
```rust
// In query_asset_info_decimals function (line 39)
AssetInfo::NativeToken { .. } => Ok(6),  // ⚠️ HARDCODED!
```

#### **Why This Is So Bad:**

- **💸 Precision Loss**: Different native tokens have different decimal places
- **💰 Financial Impact**: Incorrect decimal handling leads to wrong calculations
- **🔄 Cross-Chain Issues**: IBC tokens may have different decimals than the native chain
- **📊 Oracle Mismatch**: Oracle prices may be in different decimal formats

#### **Attack Scenarios:**

1. **Decimal Mismatch Attack**:
   ```rust
   // Attacker deposits 1,000,000 micro-tokens (6 decimals)
   // Contract thinks it's 1,000,000 base tokens (18 decimals)
   // 1,000,000 * 10^6 vs 1,000,000 * 10^18 = 10^12 difference!
   ```

2. **Cross-Chain Token Confusion**:
   ```rust
   // IBC token with 18 decimals treated as 6 decimals
   // Massive precision loss in calculations
   ```

3. **Oracle Price Manipulation**:
   ```rust
   // Oracle prices in 18 decimals, but contract uses 6
   // All price calculations become wrong
   ```

#### **Real-World Impact:**
- **💀 Fund Loss**: Users lose money due to incorrect calculations
- **🔄 Vault Freeze**: Incorrect decimal handling breaks vault operations
- **📉 Value Distortion**: Share calculations become meaningless

---

### 2. 🟠 **HIGH: Missing Asset Validation in Transfer Logic**

#### **The Problem:**
```rust
// In assert_send_asset_to_contract function (lines 44-68)
pub fn assert_send_asset_to_contract(
    info: MessageInfo,
    env: Env,
    asset: Asset,  // ⚠️ No validation of asset.amount or asset.info
) -> Result<Option<WasmMsg>, ContractError> {
    // ... no validation of asset.amount for zero or negative values
    // ... no validation of asset.info for malicious contracts
}
```

#### **Why This Is So Bad:**

- **🚫 Zero Amount Attacks**: Users can send zero amounts to trigger state changes
- **💀 Malicious Contract**: Attacker can specify malicious CW20 contracts
- **🔄 Reentrancy**: No validation of contract addresses
- **📊 State Corruption**: Invalid assets can corrupt vault state

#### **Attack Scenarios:**

1. **Zero Amount State Manipulation**:
   ```rust
   // Attacker sends Asset { amount: 0, info: valid_token }
   // Triggers state changes without actual value transfer
   ```

2. **Malicious Contract Injection**:
   ```rust
   // Attacker specifies malicious CW20 contract
   // Contract calls malicious contract during transfer
   ```

3. **Reentrancy via Asset Info**:
   ```rust
   // Malicious contract in asset.info triggers reentrancy
   // During query_asset_info_balance or other operations
   ```

---

### 3. 🟡 **MEDIUM: Insufficient Balance Validation**

#### **The Problem:**
```rust
// In assert_send_asset_to_contract for native tokens (lines 61-65)
AssetInfo::NativeToken { denom } => {
    if must_pay(&info, &denom)? < asset.amount {  // ⚠️ Only checks if sufficient, not exact
        return Err(ContractError::InsufficientFunds {});
    }
    Ok(None)
}
```

#### **Why This Is So Bad:**

- **💰 Overpayment**: Users can send more than required, funds get stuck
- **🔄 State Inconsistency**: Contract state doesn't match actual funds
- **📊 Calculation Errors**: Vault calculations become incorrect

#### **Attack Scenarios:**

1. **Overpayment Attack**:
   ```rust
   // User wants to deposit 1000 tokens
   // Sends 2000 tokens (overpayment)
   // Extra 1000 tokens get stuck in contract
   ```

2. **State Corruption**:
   ```rust
   // Contract thinks it has 1000 tokens
   // Actually has 2000 tokens
   // All calculations become wrong
   ```

---

### 4. 🟡 **MEDIUM: Missing Error Handling in Balance Queries**

#### **The Problem:**
```rust
// In query_asset_info_balance function (lines 19-28)
pub fn query_asset_info_balance(
    querier: &QuerierWrapper,
    asset_info: AssetInfo,
    addr: Addr,
) -> Result<Uint128, cosmwasm_std::StdError> {
    match asset_info {
        AssetInfo::Token { contract_addr, .. } => query_token_balance(querier, contract_addr, addr),
        AssetInfo::NativeToken { denom } => query_balance(querier, addr, denom),
    }
    // ⚠️ No error handling for contract failures or network issues
}
```

#### **Why This Is So Bad:**

- **🔄 Contract Failures**: If CW20 contract is malicious or broken, query fails
- **🌐 Network Issues**: Temporary network issues can break vault operations
- **💀 DoS Attacks**: Attacker can deploy malicious contracts to break queries

---

## 🛡️ **RECOMMENDED FIXES**

### **Fix 1: Dynamic Decimal Detection**
```rust
pub fn query_asset_info_decimals(
    querier: &QuerierWrapper,
    asset_info: AssetInfo,
) -> Result<u8, ContractError> {
    match asset_info {
        AssetInfo::Token { contract_addr, .. } => {
            let cw20::TokenInfoResponse { decimals, .. } = validate_cw20(querier, &contract_addr)?;
            Ok(decimals)
        }
        AssetInfo::NativeToken { denom } => {
            // Query the bank module for actual decimals
            let bank_info = querier.query_bank_denom_metadata(&denom)?;
            Ok(bank_info.denom_units
                .iter()
                .find(|unit| unit.denom == denom)
                .map(|unit| unit.exponent)
                .unwrap_or(6)) // Fallback to 6 if not found
        }
    }
}
```

### **Fix 2: Asset Validation**
```rust
pub fn assert_send_asset_to_contract(
    info: MessageInfo,
    env: Env,
    asset: Asset,
) -> Result<Option<WasmMsg>, ContractError> {
    // Validate asset amount
    if asset.amount.is_zero() {
        return Err(ContractError::Std(StdError::generic_err("Asset amount cannot be zero")));
    }
    
    // Validate asset info
    match &asset.info {
        AssetInfo::Token { contract_addr } => {
            // Validate that the contract is a legitimate CW20
            validate_cw20(&env.querier, contract_addr)?;
        }
        AssetInfo::NativeToken { denom } => {
            if denom.is_empty() {
                return Err(ContractError::Std(StdError::generic_err("Empty denom not allowed")));
            }
        }
    }
    
    // ... rest of the function
}
```

### **Fix 3: Exact Payment Validation**
```rust
AssetInfo::NativeToken { denom } => {
    let paid = must_pay(&info, &denom)?;
    if paid != asset.amount {
        return Err(ContractError::Std(StdError::generic_err(
            format!("Exact payment required: expected {}, got {}", asset.amount, paid)
        )));
    }
    Ok(None)
}
```

### **Fix 4: Enhanced Error Handling**
```rust
pub fn query_asset_info_balance(
    querier: &QuerierWrapper,
    asset_info: AssetInfo,
    addr: Addr,
) -> Result<Uint128, ContractError> {
    match asset_info {
        AssetInfo::Token { contract_addr, .. } => {
            query_token_balance(querier, contract_addr, addr)
                .map_err(|e| ContractError::Std(StdError::generic_err(
                    format!("Failed to query token balance: {}", e)
                )))
        }
        AssetInfo::NativeToken { denom } => {
            query_balance(querier, addr, denom)
                .map_err(|e| ContractError::Std(StdError::generic_err(
                    format!("Failed to query native balance: {}", e)
                )))
        }
    }
}
```

---

## 📊 **VULNERABILITY SEVERITY MATRIX**

| Vulnerability | Severity | Impact | Likelihood | Risk Score |
|---------------|----------|---------|------------|------------|
| Hardcoded Decimals | 🔴 Critical | 💀 Precision Loss | 🟡 Medium | **9/10** |
| Missing Asset Validation | 🟠 High | 🚫 State Corruption | 🟡 Medium | **7/10** |
| Insufficient Balance Validation | 🟡 Medium | 💰 Overpayment | 🟢 High | **6/10** |
| Missing Error Handling | 🟡 Medium | 🔄 DoS Attacks | 🟡 Medium | **5/10** |

**Risk Score = Impact × Likelihood**

---

## 🎯 **KEY TAKEAWAYS**

1. **Never hardcode decimals** - Always query the actual decimal places
2. **Validate all inputs** - Check amounts, addresses, and contract validity
3. **Handle exact payments** - Prevent overpayment and state corruption
4. **Add comprehensive error handling** - Protect against contract failures
5. **Test with different token types** - Ensure compatibility across chains

**Remember**: Asset handling is critical for vault operations. Incorrect decimal handling or missing validation can lead to massive financial losses and vault failures.
