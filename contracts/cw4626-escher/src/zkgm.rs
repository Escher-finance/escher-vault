use ucs03_zkgm::com::{
    Batch, Call, Instruction, SolverMetadata, TokenOrderV2, INSTR_VERSION_0, INSTR_VERSION_2,
    OP_BATCH, OP_CALL, OP_TOKEN_ORDER, TOKEN_ORDER_KIND_SOLVE,
};

use alloy::sol_types::SolValue;
use alloy_primitives::Bytes as AlloyBytes;
use alloy_primitives::Uint;
use cosmwasm_std::{
    to_json_binary, Addr, Binary, Coin, CosmosMsg, StdError, Uint128, Uint64, WasmMsg,
};
use ibc_union_spec::{ChannelId, Duration, Timestamp};
use std::str::FromStr;
use ucs03_zkgm;
use unionlabs_primitives::{Bytes, H256};
type AlloyUint256 = Uint<256, 4>;
use crate::error::{ContractError, ContractResult};

const TIMEOUT_OFFSET: u64 = 604_800; // 7 days period

/// # Errors
/// Will return error if message serialization or validation fails
pub fn send_token_order_v2(
    time: Timestamp,
    channel_id: u32,
    sender: &str,
    receiver: &str,
    base_token: &str,
    base_amount: Uint128,
    quote_token: &str,
    quote_amount: Uint128,
    salt: &str,
) -> ContractResult<Binary> {
    let recipient_address = validate_and_parse_address(receiver)?;
    let quote_token = validate_and_parse_address(quote_token)?;

    let metadata = SolverMetadata {
        solverAddress: Vec::from(quote_token.clone()).into(),
        metadata: alloy_primitives::Bytes::default(),
    };

    let token_order_instruction = Instruction {
        version: INSTR_VERSION_2,
        opcode: OP_TOKEN_ORDER,
        operand: TokenOrderV2 {
            sender: sender.as_bytes().to_vec().into(),
            receiver: Vec::from(recipient_address).into(),
            base_token: base_token.as_bytes().to_vec().into(),
            base_amount: AlloyUint256::from(base_amount.u128()),
            quote_token: Vec::from(quote_token).into(),
            quote_amount: AlloyUint256::from(quote_amount.u128()),
            kind: TOKEN_ORDER_KIND_SOLVE,
            metadata: metadata.abi_encode_params().into(),
        }
        .abi_encode_params()
        .into(),
    };

    let timeout_timestamp = get_timeout_timestamp_from_time(time)?;

    let salt = validate_and_parse_salt(salt)?;

    let relay_transfer_msg: ucs03_zkgm::msg::ExecuteMsg = ucs03_zkgm::msg::ExecuteMsg::Send {
        channel_id: ChannelId::from_raw(channel_id).ok_or(ContractError::InvalidChannelId {})?,
        timeout_height: Uint64::from(0u64),
        timeout_timestamp,
        salt,
        instruction: token_order_instruction.abi_encode_params().into(),
    };

    let transfer_relay_msg = to_json_binary(&relay_transfer_msg)?;
    Ok(transfer_relay_msg)
}

/// Call LST bond function via ucs03 zkgm
/// # Errors
/// Will return error if messages fail to serialize or validation fails
pub fn call_lst_bond(
    vault_contract_address: &str,
    time: Timestamp,
    channel_id: u32,
    amount: Uint128,          // it will be the amount of U that is staked
    denom: &str,              // it will be "au"
    min_mint_amount: Uint128, // it will be the expected amount of EU
    base_token: &str,         // hex base token of cw20 contract of U at babylon
    quote_token: &str,        // hex token of U (au) at Union : 0x6175
    u_solver_address: &str, // hex token of union1uuuuuuuuu9un2qpksam7rlttpxc8dc76mcphhsmp39pxjnsvrtcqvyv57r -> this is the U solver address on testnet
    salt: &str,
    proxy_account: &str, // this proxy account address is union address that represents vault on union
    zkgm_token_minter: &str, // zkgm coken minter is the contract that handle minting and burn of eU
    lst_contract_address: &str, // union liquid staking contract address
    lst_base_token: &str, // eU CW20 contract address on union (because it will send back from Union to Babylon so it is named LST base token)
    lst_quote_token: &str, // eU CW20 contract address on babylon
    babylon_ucs03_zkgm: &str, // UCS03 zkgm contract address on Babylon
    union_ucs03_zkgm: &str, // UCS03 zkgm contract address on Union
) -> ContractResult<CosmosMsg> {
    let proxy_account_address = validate_and_parse_address(proxy_account)?;
    let quote_token = validate_and_parse_address(quote_token)?;

    let sender_bytes: AlloyBytes = vault_contract_address.as_bytes().to_vec().into();

    //let cw_account_address = call_ucs03_to_get_predicted_adderss_here;

    let metadata = SolverMetadata {
        solverAddress: Vec::from(u_solver_address).into(),
        metadata: AlloyBytes::default(),
    };

    // Create token order v2 to send U from babylon vault to union LST
    let token_order_v2 = Instruction {
        version: INSTR_VERSION_2,
        opcode: OP_TOKEN_ORDER,
        operand: TokenOrderV2 {
            sender: sender_bytes.clone(), // vault is the sender
            receiver: Vec::from(proxy_account_address.clone()).into(),
            base_token: base_token.as_bytes().to_vec().into(),
            base_amount: AlloyUint256::from(amount.u128()),
            quote_token: Vec::from(quote_token).into(),
            quote_amount: AlloyUint256::from(amount.u128()),
            kind: TOKEN_ORDER_KIND_SOLVE,
            metadata: metadata.abi_encode_params().into(),
        }
        .abi_encode_params()
        .into(),
    };

    let channel_id = ChannelId::from_raw(channel_id).ok_or(ContractError::InvalidChannelId {})?;

    let timeout_timestamp = get_timeout_timestamp_from_time(time)?;

    let salt: unionlabs_primitives::H256 = match unionlabs_primitives::H256::from_str(salt) {
        Ok(s) => s,
        Err(e) => {
            return Err(ContractError::Std(StdError::generic_err(format!(
                "failed to parse salt: {salt}, reason: {e}"
            ))))
        }
    };

    // Generate 3 payloads as array/vector: bond, increase_allowance and send back via tokenorderv2, these need to be encoded as Bytes
    let contract_calldata = generate_bond_calldata(
        amount,
        denom,
        vault_contract_address,
        min_mint_amount,
        proxy_account,
        zkgm_token_minter,
        channel_id,
        timeout_timestamp,
        salt,
        lst_contract_address,
        lst_base_token,
        lst_quote_token,
        union_ucs03_zkgm, // it will be executed on Union so it need union ucs03 zkgm
    )?;

    let call_instruction: Instruction = Instruction {
        version: INSTR_VERSION_0,
        opcode: OP_CALL,
        operand: Call {
            sender: sender_bytes,
            eureka: false,
            contract_address: Vec::from(proxy_account_address).into(),
            contract_calldata: contract_calldata.into(),
        }
        .abi_encode_params()
        .into(),
    };

    let batch_instruction = Instruction {
        version: INSTR_VERSION_0,
        opcode: OP_BATCH,
        operand: Batch {
            instructions: vec![token_order_v2, call_instruction],
        }
        .abi_encode_params()
        .into(),
    };

    let ucs03_send_msg = ucs03_zkgm::msg::ExecuteMsg::Send {
        channel_id,
        timeout_height: Uint64::from(0u64),
        timeout_timestamp,
        salt,
        instruction: batch_instruction.abi_encode_params().into(),
    };

    let send_msg = to_json_binary(&ucs03_send_msg)?;
    let execute_increase_allowance_msg: CosmosMsg = WasmMsg::Execute {
        contract_addr: babylon_ucs03_zkgm.to_string(),
        msg: to_json_binary(&send_msg)?,
        funds: vec![],
    }
    .into();

    Ok(execute_increase_allowance_msg)
}

/// Sends call instruction to ucs03 and return cosmos msg execute call to ucs03
///
/// # Errors
/// Will return error if validation or message serialization fails
pub fn ucs03_call_lst(
    sender: &str,
    channel_id: u32,
    time: Timestamp,
    contract_address: &str,
    contract_calldata: Bytes,
    salt: H256,
) -> ContractResult<Binary> {
    let contract_address = validate_and_parse_address(contract_address)?;

    let call_instruction = Instruction {
        version: INSTR_VERSION_0,
        opcode: OP_CALL,
        operand: Call {
            sender: sender.as_bytes().to_vec().into(),
            eureka: false,
            contract_address: contract_address.into(),
            contract_calldata: contract_calldata.into(),
        }
        .abi_encode_params()
        .into(),
    };

    let timeout_timestamp = get_timeout_timestamp_from_time(time)?;

    let ucs03_send_msg: ucs03_zkgm::msg::ExecuteMsg = ucs03_zkgm::msg::ExecuteMsg::Send {
        channel_id: ChannelId::from_raw(channel_id).ok_or(ContractError::InvalidChannelId {})?,
        timeout_height: Uint64::from(0u64),
        timeout_timestamp,
        salt,
        instruction: call_instruction.abi_encode_params().into(),
    };

    let ucc03_msg_bin = to_json_binary(&ucs03_send_msg)?;
    Ok(ucc03_msg_bin)
}

/// # Errors
/// Will return error if overflow
pub fn get_timeout_timestamp_from_time(time: Timestamp) -> ContractResult<Timestamp> {
    let duration_offset = Duration::from_secs(TIMEOUT_OFFSET);
    Ok(Timestamp::from_nanos(
        time.plus_duration(duration_offset)
            .ok_or(ContractError::TimestampOverflow {})?
            .as_nanos(),
    ))
}

/// Validates and returns `salt` as `unionlabs_primitives::H256` for usage with ZKGM
///
/// # Errors
/// Will return error if validations fail
pub fn validate_and_parse_salt(salt: &str) -> ContractResult<unionlabs_primitives::H256> {
    let hex = salt
        .strip_prefix("0x")
        .ok_or(ContractError::InvalidSalt {})?;
    unionlabs_primitives::H256::from_str(hex).map_err(|_| ContractError::InvalidSalt {})
}

/// Validates and returns `address` as `Bytes` for usage with ZKGM
///
/// # Errors
/// Will return error if validations fail
pub fn validate_and_parse_address(address: &str) -> ContractResult<Bytes> {
    let hex = address
        .strip_prefix("0x")
        .ok_or(ContractError::InvalidHexAddress {})?;
    Bytes::from_str(hex).map_err(|_| ContractError::InvalidHexAddress {})
}

/// # Errors
/// Will return error if validations fail
pub fn generate_bond_calldata(
    amount: Uint128,
    denom: &str, // it will be au string
    vault_contract_address: &str,
    min_mint_amount: Uint128,
    proxy_account_address: &str,
    zkgm_token_minter: &str,
    channel_id: ChannelId,
    timeout: Timestamp,
    salt: unionlabs_primitives::H256,
    lst_contract_address: &str, // liquid staking contract address on Union
    lst_base_token: &str,       // eU contract address on union
    lst_quote_token: &str,      // eU contract address on babylon
    union_ucs03_zkgm: &str,     // ucs03 zkgm contract address on union
) -> ContractResult<Bytes> {
    let mut call_msgs: Vec<CosmosMsg> = vec![];

    // 1. construct bond msg call to LST
    let bond_msg = lst::msg::ExecuteMsg::Bond {
        mint_to_address: Addr::unchecked(proxy_account_address),
        min_mint_amount,
    };
    let execute_bond_msg = WasmMsg::Execute {
        contract_addr: lst_contract_address.to_string(),
        msg: to_json_binary(&bond_msg)?,
        funds: vec![Coin {
            denom: denom.to_string(),
            amount,
        }],
    };
    call_msgs.push(execute_bond_msg.into());

    // 2. construct increase allowance to zkgm token minter
    let increase_allowance_msg = cw20::Cw20ExecuteMsg::IncreaseAllowance {
        spender: zkgm_token_minter.to_string(),
        amount: min_mint_amount,
        expires: None,
    };
    let execute_increase_allowance_msg = WasmMsg::Execute {
        contract_addr: lst_base_token.to_string(), // it is eU contract adddress on Union
        msg: to_json_binary(&increase_allowance_msg)?,
        funds: vec![],
    };
    call_msgs.push(execute_increase_allowance_msg.into());

    let metadata = SolverMetadata {
        solverAddress: Vec::from(lst_quote_token).into(),
        metadata: alloy_primitives::Bytes::default(),
    };

    // 3. construct token order v2 to send back from to sender
    let fungible_order_instruction = Instruction {
        version: INSTR_VERSION_2,
        opcode: OP_TOKEN_ORDER,
        operand: TokenOrderV2 {
            sender: Vec::from(proxy_account_address).into(),
            receiver: Vec::from(vault_contract_address).into(),
            base_token: Vec::from(lst_base_token).into(), // eU contract address on union
            base_amount: AlloyUint256::from(amount.u128()),
            quote_token: Vec::from(lst_quote_token).into(), // eU contract address on babylon
            quote_amount: AlloyUint256::from(amount.u128()),
            kind: TOKEN_ORDER_KIND_SOLVE,
            metadata: metadata.abi_encode_params().into(),
        }
        .abi_encode_params()
        .into(),
    };

    let send_msg = ucs03_zkgm::msg::ExecuteMsg::Send {
        channel_id,
        timeout_height: Uint64::from(0u64),
        timeout_timestamp: timeout,
        salt,
        instruction: fungible_order_instruction.abi_encode_params().into(),
    };

    let execute_send_msg = WasmMsg::Execute {
        contract_addr: union_ucs03_zkgm.to_string(), // UCS03 contract on Union
        msg: to_json_binary(&send_msg)?,
        funds: vec![],
    };

    call_msgs.push(execute_send_msg.into());

    let msgs_bin = to_json_binary(&call_msgs)?;
    let msgs_bytes: Bytes = msgs_bin.to_vec().into();
    Ok(msgs_bytes)
}

// generate 3 payload inside 1 call
// First:
//     {
//      {
//      "wasm": {
//       "execute": {
//         "contract_addr": "union1d2r4ecsuap4pujrlf3nz09vz8eha8y0z25knq0lfxz4yzn83v6kq0jxsmk",
//         "msg": "eyJib25kIjp7Im1pbnRfdG9fYWRkcmVzcyI6InVuaW9uMWNubnR1MDlqM3M0OWtxc3ByN3N0NXJwYXR2bjY4aDR5dWVhcGpza2M5c3VnOHQybXU3eXM0NnRmcTkiLCJtaW5fbWludF9hbW91bnQiOiI5MDAwMDAwMDAwMDAwMDAwMDAifX0=",
//         "funds": [
//           {
//             "denom": "au",
//             "amount": "1000000000000000000"
//           }
//         ]
//       }
//     }
//   },
//   "bond":
//      {
//          "mint_to_address": "union1p20079kpv9mm4xj7cu3wujluesclfjsf5335he7vg7pxm69vvxksjm3yq8", // mint to cw account contract
//          "min_mint_amount": "900000000000000000"
//          }
//      }
//
//     Second: is to increase allowance
//     {
//      "increase_allowance": {
//          "spender": "union1t5awl707x54k6yyx7qfkuqp890dss2pqgwxh07cu44x5lrlvt4rs8hqmk0", // spender is zkgm token minter
//          "amount": "900000000000000000"
//      }
//
//  Third: tokenorderv2 instruction to send back
//
// {
//   "send": {
//     "channel_id": 20,
//     "timeout_height": "0",
//     "timeout_timestamp": "1757706966815000000",
//     "salt": "0xc9e38eb588b3d0247a6ab988225f4cbab7c1c4687a6f9830da3d5132c01bc4a6",
//     "instruction": "0x00000000
// }

// Function to get contract calldata of 3 payloads that should be generated to do bond on remote LST
// pub fn get_bond_contract_calldata(
//     min_mint_amount: Uint128,
//     cw_account: String,
// ) -> Result<Bytes, ContractError> {
//     let bond_msg = lst::msg::ExecuteMsg::Bond {
//         mint_to_address: cw_account,
//         min_mint_amount,
//     };

//     let msg_bytes: Bytes = to_json_binary(bond_msg).to_vec().into();
//     Ok(msg_bytes)
// }
