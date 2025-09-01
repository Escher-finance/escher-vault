# Native Token Support Modifications

## Overview
This document summarizes the modifications made to the CW4626 vault contracts to support native tokens (like `ubbn`) in addition to CW20 tokens.

## Problem Statement
The original contracts were designed only for CW20 tokens, but you needed to test with native `ubbn` tokens. This required significant modifications to:
1. Support both CW20 and native token types
2. Handle native token deposits and withdrawals
3. Maintain backward compatibility
4. Add proper error handling for native operations

## Files Modified

### 1. packages/cw4626/src/msg.rs
- **Added**: `UnderlyingToken` enum with `Cw20` and `Native` variants
- **Updated**: `Cw4626InstantiateMsg` to use the new enum
- **Added**: Legacy support with `Cw4626InstantiateMsgLegacy`
- **Added**: Native token execute messages:
  - `DepositNative`
  - `MintNative`
  - `WithdrawNative`
  - `RedeemNative`

### 2. contracts/cw4626-base/src/state.rs
- **Added**: `TokenType` enum matching the package structure
- **Added**: `TOKEN_TYPE` storage item
- **Kept**: `UNDERLYING_ASSET` for backward compatibility

### 3. contracts/cw4626-base/src/error.rs
- **Added**: `InsufficientFunds` error for native token operations
- **Added**: `InvalidTokenType` error for type mismatches

### 4. contracts/cw4626-base/src/helpers.rs
- **Added**: `_deposit_native` function for native token deposits
- **Added**: `_withdraw_native` function for native token withdrawals
- **Updated**: Imports to include `BankMsg` and `Coin`

### 5. contracts/cw4626-base/src/execute.rs
- **Added**: `deposit_native` execute function
- **Added**: `mint_native` execute function
- **Added**: `withdraw_native` execute function
- **Added**: `redeem_native` execute function
- **Kept**: All original CW20 functions for backward compatibility

### 6. contracts/cw4626-base/src/contract.rs
- **Updated**: `instantiate` function to handle both token types
- **Updated**: `execute` function to route native token messages
- **Added**: Logic to save token type and handle native vs CW20 differently

## Key Features

### ✅ Native Token Support
- Direct deposit of native `ubbn` tokens
- Direct withdrawal of native `ubbn` tokens
- No need for CW20 wrapper contracts

### ✅ Backward Compatibility
- All existing CW20 functionality preserved
- Legacy instantiate messages still supported
- Existing deployments continue to work

### ✅ Enhanced Error Handling
- Specific errors for insufficient funds
- Type validation for native vs CW20 operations
- Clear error messages for debugging

### ✅ Flexible Token Types
- Support for any native token denom
- Support for any CW20 token address
- Runtime token type detection

## Usage Examples

### Instantiate with Native Ubbn
```json
{
  "underlying_token": {
    "native": {
      "denom": "ubbn"
    }
  },
  "share_name": "Native Ubbn Vault",
  "share_symbol": "nUBBN",
  "share_marketing": {
    "project": "https://your-project.com",
    "description": "Vault for native ubbn tokens",
    "marketing": "bbn1..."
  }
}
```

### Instantiate with CW20 Token (Legacy)
```json
{
  "underlying_token": {
    "cw20": {
      "address": "bbn1..."
    }
  },
  "share_name": "CW20 Token Vault",
  "share_symbol": "vTOKEN",
  "share_marketing": {
    "project": "https://your-project.com",
    "description": "Vault for CW20 token",
    "marketing": "bbn1..."
  }
}
```

## New Execute Messages

### Deposit Native Tokens
```bash
babylond tx wasm execute <contract-address> '{
  "deposit_native": {
    "receiver": "bbn1..."
  }
}' --from <key> --amount 1000000ubbn
```

### Withdraw Native Tokens
```bash
babylond tx wasm execute <contract-address> '{
  "withdraw_native": {
    "assets": "1000000",
    "receiver": "bbn1...",
    "owner": "bbn1..."
  }
}' --from <key>
```

## Testing

### Build Verification
```bash
# Build base contract
cargo wasm -p cw4626-base

# Build escher contract
cargo wasm -p cw4626-escher

# Run test script
./scripts/test-native-tokens.sh
```

### Deployment
```bash
# Deploy native ubbn version
./scripts/deploy-native-ubbn.sh
```

## Benefits

1. **Direct Native Token Support**: No need for CW20 wrappers
2. **Simplified Testing**: Test directly with ubbn tokens
3. **Better User Experience**: Users can deposit native tokens directly
4. **Flexible Architecture**: Support for both token types in one contract
5. **Future-Proof**: Easy to add support for other native tokens

## Next Steps

1. **Deploy Modified Contract**: Use the deployment script
2. **Test Native Operations**: Verify deposit/withdraw work correctly
3. **Test Share Tokens**: Ensure minting/burning works as expected
4. **Integration Testing**: Test with your pool and other contracts
5. **Production Deployment**: Deploy to mainnet when ready

## Notes

- The contract automatically detects token type at instantiation
- Native tokens use 6 decimals by default (configurable)
- All existing CW20 functionality remains unchanged
- Error handling is comprehensive for both token types
- Gas costs for native operations are optimized

## Support

For issues or questions about the native token modifications:
1. Check the contract compilation output
2. Verify the instantiate message format
3. Test with small amounts first
4. Review the error messages for debugging
