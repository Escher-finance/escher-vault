use ucs03_zkgm::com::{
    Batch, Call, Instruction, SolverMetadata, TokenOrderV2, INSTR_VERSION_0, INSTR_VERSION_2,
    OP_BATCH, OP_CALL, OP_TOKEN_ORDER, TOKEN_ORDER_KIND_SOLVE,
};

use crate::{error::ContractResult, ContractError};
use alloy::sol_types::SolValue;
use alloy_primitives::{Bytes, Uint};
use cosmwasm_std::{to_json_binary, Binary, StdError, Uint128, Uint64};
use ibc_union_spec::{ChannelId, Duration, Timestamp};
use std::str::FromStr;
use ucs03_zkgm;
use unionlabs_primitives::H256;
type AlloyUint256 = Uint<256, 4>;

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
        metadata: Bytes::default(),
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

    let salt: unionlabs_primitives::H256 = match unionlabs_primitives::H256::from_str(salt) {
        Ok(s) => s,
        Err(e) => {
            return Err(ContractError::Std(StdError::generic_err(format!(
                "failed to parse salt: {salt}, reason: {e}"
            ))))
        }
    };

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

/// # Errors
/// Will return error if messages fail to serialize or validation fails
pub fn send_token_order_v2_and_call_lst(
    time: Timestamp,
    channel_id: u32,
    sender: &str,
    base_token: &str,
    base_amount: Uint128,
    quote_token: &str,
    quote_amount: Uint128,
    salt: &str,
    proxy_account_address: &str,
    contract_calldata: Bytes,
) -> ContractResult<Binary> {
    let proxy_account_address = validate_and_parse_address(proxy_account_address)?;
    let quote_token = validate_and_parse_address(quote_token)?;

    let sender_bytes = sender.as_bytes().to_vec().into();

    //let cw_account_address = call_ucs03_to_get_predicted_adderss_here;

    let metadata = SolverMetadata {
        solverAddress: Vec::from(quote_token.clone()).into(),
        metadata: Bytes::default(),
    };

    let fungible_order_instruction = Instruction {
        version: INSTR_VERSION_2,
        opcode: OP_TOKEN_ORDER,
        operand: TokenOrderV2 {
            sender: sender_bytes,
            receiver: Vec::from(proxy_account_address.clone()).into(),
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

    let call_instruction: Instruction = Instruction {
        version: INSTR_VERSION_0,
        opcode: OP_CALL,
        operand: Call {
            sender: sender.as_bytes().to_vec().into(),
            eureka: false,
            contract_address: Vec::from(proxy_account_address).into(),
            contract_calldata,
        }
        .abi_encode_params()
        .into(),
    };

    let batch_instruction = Instruction {
        version: INSTR_VERSION_0,
        opcode: OP_BATCH,
        operand: Batch {
            instructions: vec![fungible_order_instruction, call_instruction],
        }
        .abi_encode_params()
        .into(),
    };

    let timeout_timestamp = get_timeout_timestamp_from_time(time)?;

    let salt: unionlabs_primitives::H256 = match unionlabs_primitives::H256::from_str(salt) {
        Ok(s) => s,
        Err(e) => {
            return Err(ContractError::Std(StdError::generic_err(format!(
                "failed to parse salt: {salt}, reason: {e}"
            ))))
        }
    };

    let relay_transfer_msg = ucs03_zkgm::msg::ExecuteMsg::Send {
        channel_id: ChannelId::from_raw(channel_id).ok_or(ContractError::InvalidChannelId {})?,
        timeout_height: Uint64::from(0u64),
        timeout_timestamp,
        salt,
        instruction: batch_instruction.abi_encode_params().into(),
    };

    let transfer_relay_msg = to_json_binary(&relay_transfer_msg)?;
    Ok(transfer_relay_msg)
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
            contract_address,
            contract_calldata,
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

// generate 3 payload inside 1 call
// First:
//     {
//   "bond":
//      {
//          "mint_to_address": "union1p20079kpv9mm4xj7cu3wujluesclfjsf5335he7vg7pxm69vvxksjm3yq8", // mint to cw account contract
//          "min_mint_amount": "900000000000000000"
//          }
//      }
//
//     Second: is to incrase allowance
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
