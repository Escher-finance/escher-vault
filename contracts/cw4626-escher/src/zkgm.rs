use ucs03_zkgm::com::{
    Batch, Call, Instruction, SolverMetadata, TokenOrderV2, INSTR_VERSION_0, INSTR_VERSION_2,
    OP_BATCH, OP_CALL, OP_TOKEN_ORDER, TOKEN_ORDER_KIND_SOLVE,
};

use crate::ContractError;
use alloy::sol_types::SolValue;
use alloy_primitives::{Bytes, Uint};
use cosmwasm_std::{to_json_binary, Binary, StdError, Uint128, Uint64};
use ibc_union_spec::{ChannelId, Duration, Timestamp};
use lst::msg::ExecuteMsg as LstExecutMsg;
use std::str::FromStr;
use ucs03_zkgm;
use unionlabs_primitives::H256;

type AlloyUint256 = Uint<256, 4>;

const TIMEOUT_OFFSET: u64 = 604800; // 1 day period

pub fn send_token_order_v2(
    time: Timestamp,
    channel_id: u32,
    sender: String,
    receiver: String,
    base_token: String,
    base_amount: Uint128,
    quote_token: String,
    quote_amount: Uint128,
    salt: String,
) -> Result<Binary, ContractError> {
    let recipient_address = match Bytes::from_str(receiver.as_str()) {
        Ok(rec) => rec,
        Err(_) => {
            return Err(ContractError::InvalidAddress {
                kind: "recipient".into(),
                address: receiver,
                reason: "address must be in hex and starts with 0x".to_string(),
            })
        }
    };
    let quote_token = match Bytes::from_str(quote_token.as_str()) {
        Ok(token) => token,
        Err(_) => {
            return Err(ContractError::InvalidAddress {
                kind: "quote_token".into(),
                address: quote_token,
                reason: "address must be in hex and starts with 0x".to_string(),
            })
        }
    };

    let metadata = SolverMetadata {
        solverAddress: Vec::from(quote_token.clone()).into(),
        metadata: Default::default(),
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

    let timeout_timestamp = get_timeout_timestamp_from_time(time);

    let salt: unionlabs_primitives::H256 = match unionlabs_primitives::H256::from_str(salt.as_str())
    {
        Ok(s) => s,
        Err(e) => {
            return Err(ContractError::Std(StdError::generic_err(format!(
                "failed to parse salt: {}, reason: {}",
                salt, e
            ))))
        }
    };

    let relay_transfer_msg: ucs03_zkgm::msg::ExecuteMsg = ucs03_zkgm::msg::ExecuteMsg::Send {
        channel_id: ChannelId::from_raw(channel_id).unwrap(),
        timeout_height: Uint64::from(0u64),
        timeout_timestamp,
        salt,
        instruction: token_order_instruction.abi_encode_params().into(),
    };

    let transfer_relay_msg = to_json_binary(&relay_transfer_msg)?;
    Ok(transfer_relay_msg)
}

pub fn send_token_order_v2_and_call_lst(
    time: Timestamp,
    channel_id: u32,
    sender: String,
    base_token: String,
    base_amount: Uint128,
    quote_token: String,
    quote_amount: Uint128,
    salt: String,
    hub_contract: String,
    contract_msg: LstExecutMsg,
) -> Result<Binary, ContractError> {
    let recipient_address = match Bytes::from_str(hub_contract.as_str()) {
        Ok(rec) => rec,
        Err(_) => {
            return Err(ContractError::InvalidAddress {
                kind: "recipient".into(),
                address: hub_contract,
                reason: "address must be in hex and starts with 0x".to_string(),
            })
        }
    };
    let quote_token = match Bytes::from_str(quote_token.as_str()) {
        Ok(token) => token,
        Err(_) => {
            return Err(ContractError::InvalidAddress {
                kind: "quote_token".into(),
                address: quote_token,
                reason: "address must be in hex and starts with 0x".to_string(),
            })
        }
    };

    let metadata = SolverMetadata {
        solverAddress: Vec::from(quote_token.clone()).into(),
        metadata: Default::default(),
    };

    let fungible_order_instruction = Instruction {
        version: INSTR_VERSION_2,
        opcode: OP_TOKEN_ORDER,
        operand: TokenOrderV2 {
            sender: sender.as_bytes().to_vec().into(),
            receiver: Vec::from(recipient_address.clone()).into(),
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
            contract_address: Vec::from(recipient_address).into(),
            contract_calldata: to_json_binary(&contract_msg)?.to_vec().into(),
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

    let timeout_timestamp = get_timeout_timestamp_from_time(time);

    let salt: unionlabs_primitives::H256 = match unionlabs_primitives::H256::from_str(salt.as_str())
    {
        Ok(s) => s,
        Err(e) => {
            return Err(ContractError::Std(StdError::generic_err(format!(
                "failed to parse salt: {}, reason: {}",
                salt, e
            ))))
        }
    };

    let relay_transfer_msg = ucs03_zkgm::msg::ExecuteMsg::Send {
        channel_id: ChannelId::from_raw(channel_id).unwrap(),
        timeout_height: Uint64::from(0u64),
        timeout_timestamp,
        salt,
        instruction: batch_instruction.abi_encode_params().into(),
    };

    let transfer_relay_msg = to_json_binary(&relay_transfer_msg)?;
    Ok(transfer_relay_msg)
}

pub fn ucs03_call_lst(
    sender: String,
    channel_id: u32,
    time: Timestamp,
    hub_contract: String,
    contract_msg: LstExecutMsg,
    salt: H256,
) -> Result<Binary, ContractError> {
    let contract_address = match Bytes::from_str(hub_contract.as_str()) {
        Ok(rec) => rec,
        Err(_) => {
            return Err(ContractError::InvalidAddress {
                kind: "recipient".into(),
                address: hub_contract,
                reason: "address must be in hex and starts with 0x".to_string(),
            })
        }
    };

    let call_instruction = Instruction {
        version: INSTR_VERSION_0,
        opcode: OP_CALL,
        operand: Call {
            sender: sender.as_bytes().to_vec().into(),
            eureka: false,
            contract_address,
            contract_calldata: to_json_binary(&contract_msg)?.to_vec().into(),
        }
        .abi_encode_params()
        .into(),
    };

    let timeout_timestamp = get_timeout_timestamp_from_time(time);

    let ucs03_send_msg: ucs03_zkgm::msg::ExecuteMsg = ucs03_zkgm::msg::ExecuteMsg::Send {
        channel_id: ChannelId::from_raw(channel_id).unwrap(),
        timeout_height: Uint64::from(0u64),
        timeout_timestamp,
        salt,
        instruction: call_instruction.abi_encode_params().into(),
    };

    let ucc03_msg_bin = to_json_binary(&ucs03_send_msg)?;
    Ok(ucc03_msg_bin)
}

pub fn get_timeout_timestamp_from_time(time: Timestamp) -> Timestamp {
    let duration_offset = Duration::from_secs(TIMEOUT_OFFSET);
    Timestamp::from_nanos(time.plus_duration(duration_offset).unwrap().as_nanos())
}
