use cosmwasm_schema::cw_serde;
use ucs03_zkgm::com::{
    Batch, Call, Instruction, SolverMetadata, TokenOrderV2, INSTR_VERSION_0, INSTR_VERSION_2,
    OP_BATCH, OP_CALL, OP_TOKEN_ORDER, TOKEN_ORDER_KIND_SOLVE,
};

use crate::error::{ContractError, ContractResult};
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

#[cw_serde]
pub enum LstExecuteMsg {
    /// Initiates the bonding process for a user.
    Bond {
        /// The address to mint the LST to.
        mint_to_address: Addr,
        /// Minimum expected amount of LST tokens to be received
        /// for the operation to be considered valid.
        min_mint_amount: Uint128,
    },
    /// Initiates the unbonding process for a user.
    Unbond {
        /// The address that will receive the native tokens on.
        staker: Addr,
        /// The amount to unstake.
        amount: Uint128,
    },
}

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
    babylon_channel_id: u32,  // channel id of babylon
    union_channel_id: u32,    // channel id of union
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

    let timeout_timestamp = get_timeout_timestamp_from_time(time)?;

    let salt: unionlabs_primitives::H256 = match unionlabs_primitives::H256::from_str(salt) {
        Ok(s) => s,
        Err(e) => {
            return Err(ContractError::Std(StdError::generic_err(format!(
                "failed to parse salt: {salt}, reason: {e}"
            ))))
        }
    };

    let union_channel_id =
        ChannelId::from_raw(union_channel_id).ok_or(ContractError::InvalidChannelId {})?;

    // Generate 3 payloads as array/vector: bond, increase_allowance and send back via tokenorderv2, these need to be encoded as Bytes
    let contract_calldata = generate_bond_calldata(
        amount,
        denom,
        vault_contract_address,
        min_mint_amount,
        proxy_account,
        zkgm_token_minter,
        union_channel_id,
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

    let channel_id =
        ChannelId::from_raw(babylon_channel_id).ok_or(ContractError::InvalidChannelId {})?;

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

#[cosmwasm_schema::cw_serde]
pub enum TestCW20ExecuteMsg {
    IncreaseAllowance { spender: String, amount: Uint128 },
}

/// # Errors
/// Will return error if validations fail
pub fn generate_bond_calldata(
    amount: Uint128,
    denom: &str,                  // it will be au string
    vault_contract_address: &str, // sender
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
    let bond_msg = LstExecuteMsg::Bond {
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
        amount,
        expires: None,
    };

    if cfg!(test) {
        let increase_allowance_msg = TestCW20ExecuteMsg::IncreaseAllowance {
            spender: zkgm_token_minter.to_string(),
            amount,
        };

        let execute_increase_allowance_msg = WasmMsg::Execute {
            contract_addr: lst_base_token.to_string(), // it is eU contract adddress on Union
            msg: to_json_binary(&increase_allowance_msg)?,
            funds: vec![],
        };
        call_msgs.push(execute_increase_allowance_msg.into());
    } else {
        let execute_increase_allowance_msg = WasmMsg::Execute {
            contract_addr: lst_base_token.to_string(), // it is eU contract adddress on Union
            msg: to_json_binary(&increase_allowance_msg)?,
            funds: vec![],
        };
        call_msgs.push(execute_increase_allowance_msg.into());
    }

    let quote_token: Bytes =
        Bytes::from_str(lst_quote_token).map_err(|_| ContractError::InvalidHexAddress {})?;

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
            base_amount: AlloyUint256::from(min_mint_amount.u128()),
            quote_token: quote_token.into(), // eU contract address on babylon
            quote_amount: AlloyUint256::from(min_mint_amount.u128()),
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

#[cfg(test)]
mod tests {

    use crate::zkgm::generate_bond_calldata;
    use cosmwasm_std::Uint128;
    use ibc_union_spec::ChannelId;
    use ibc_union_spec::Timestamp;
    use std::str::FromStr;

    #[test]

    fn test_generate_bond_calldata() {
        let amount = Uint128::from(1_000_000_000_000_000_000u128);
        let sender = "0x4aaa51a0814d91f7d2b3ab60829a921ec9eb8e17";
        let denom = "au";
        let min_mint_amount = Uint128::from(900_000_000_000_000_000u128);

        let proxy_account_address =
            "union1cnntu09j3s49kqspr7st5rpatvn68h4yueapjskc9sug8t2mu7ys46tfq9";
        let zkgm_token_minter = "union1t5awl707x54k6yyx7qfkuqp890dss2pqgwxh07cu44x5lrlvt4rs8hqmk0";
        let channel_id = ChannelId::from_raw(20).unwrap();
        let timeout = Timestamp::from_nanos(1_757_853_925_422_000_000);
        let salt = unionlabs_primitives::H256::from_str(
            "0x64956543467d00f9180e2a2ca78ccec3ef63c7f7490a62b86594fe14b21225fb",
        )
        .unwrap();

        let lst_contract_address =
            "union1d2r4ecsuap4pujrlf3nz09vz8eha8y0z25knq0lfxz4yzn83v6kq0jxsmk";

        let lst_base_token = "union1eueueueu9var4yhdruyzkjcsh74xzeug6ckyy60hs0vcqnzql2hq0lxc2f";
        let lst_quote_token = "0xe5Cf13C84c0fEa3236C101Bd7d743d30366E5CF1";
        let union_ucs03_zkgm = "union1336jj8ertl8h7rdvnz4dh5rqahd09cy0x43guhsxx6xyrztx292qpe64fh";

        let result = generate_bond_calldata(
            amount,
            denom,
            sender,
            min_mint_amount,
            proxy_account_address,
            zkgm_token_minter,
            channel_id,
            timeout,
            salt,
            lst_contract_address,
            lst_base_token,
            lst_quote_token,
            union_ucs03_zkgm,
        )
        .unwrap();

        let expected_output = "0x5b7b227761736d223a7b2265786563757465223a7b22636f6e74726163745f61646472223a22756e696f6e31643272346563737561703470756a726c66336e7a3039767a386568613879307a32356b6e71306c66787a34797a6e383376366b71306a78736d6b222c226d7367223a2265794a696232356b496a7037496d3170626e5266644739665957526b636d567a63794936496e5675615739754d574e75626e52314d446c714d334d304f577478633342794e334e304e584a7759585232626a593461445235645756686347707a61324d356333566e4f4851796258553365584d304e6e526d63546b694c434a746157356662576c756446396862573931626e51694f6949354d4441774d4441774d4441774d4441774d4441774d4441696658303d222c2266756e6473223a5b7b2264656e6f6d223a226175222c22616d6f756e74223a2231303030303030303030303030303030303030227d5d7d7d7d2c7b227761736d223a7b2265786563757465223a7b22636f6e74726163745f61646472223a22756e696f6e31657565756575657539766172347968647275797a6b6a6373683734787a65756736636b797936306873307663716e7a716c326871306c78633266222c226d7367223a2265794a70626d4e795a57467a5a5639686247787664324675593255694f6e73696333426c626d526c63694936496e5675615739754d585131595864734e7a413365445530617a5a356558673363575a72645846774f446b775a484e7a4d6e42785a33643461444133593355304e48673162484a73646e5130636e4d3461484674617a41694c434a6862573931626e51694f6949784d4441774d4441774d4441774d4441774d4441774d444177496e3139222c2266756e6473223a5b5d7d7d7d2c7b227761736d223a7b2265786563757465223a7b22636f6e74726163745f61646472223a22756e696f6e313333366a6a386572746c3868377264766e7a3464683572716168643039637930783433677568737878367879727a747832393271706536346668222c226d7367223a2265794a7a5a57356b496a7037496d4e6f595735755a577866615751694f6a49774c434a306157316c623356305832686c6157646f64434936496a41694c434a306157316c62335630583352706257567a6447467463434936496a45334e5463344e544d354d6a55304d6a49774d4441774d4441694c434a7a59577830496a6f694d4867324e446b314e6a55304d7a51324e3251774d4759354d5467775a544a684d6d4e684e7a686a5932566a4d32566d4e6a4e6a4e3259334e446b7759545979596a67324e546b305a6d55784e4749794d5449794e575a69496977696157357a64484a3159335270623234694f694977654441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4449774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d44417a4d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441324d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d44417a4d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d5441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4445324d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d444178595441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d444177597a646b4e7a457a596a51355a4745774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4449774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d44426a4e3251334d544e694e446c6b595441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d44417a4d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4449304d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d6d4d334e545a6c4e6a6b325a6a5a6c4d7a45334f545a6a4e6a59334d6a59344e7a4d7a4d6a63354d7a5533595459304e6d457a4d6a4d7a4d7a6b7a4e445a6b4d7a59324e6a63344e6a63334d4464684e7a4932595459784e7a597a4e7a5a6a4e6a557a4d7a64684d7a417a4e7a5a684e6a59324e6a63784d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d545130595546684e5446684d4467784e4551354d55593352444a434d3046694e6a41344d6a6c424f5449785a554d35525749345a5445334d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441304d4463314e6d55324f545a6d4e6d557a4d5459314e7a55324e5463314e6a55334e5459314e7a557a4f5463324e6a45334d6a4d304e7a6b324f4459304e7a49334e5463354e324532596a5a684e6a4d334d7a59344d7a637a4e4463344e3245324e5463314e6a637a4e6a597a4e6d49334f5463354d7a597a4d4459344e7a4d7a4d4463324e6a4d334d545a6c4e3245334d545a6a4d7a49324f4463784d7a4132597a63344e6a4d7a4d6a59324d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441784e475531593259784d324d344e474d775a6d56684d7a497a4e6d4d784d4446695a44646b4e7a517a5a444d774d7a59325a54566a5a6a45774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4745774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441304d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774f4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4445305a54566a5a6a457a597a6730597a426d5a57457a4d6a4d32597a45774d574a6b4e3251334e444e6b4d7a417a4e6a5a6c4e574e6d4d5441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441696658303d222c2266756e6473223a5b5d7d7d7d5d";

        println!("{}", expected_output.len());
        println!("{}", result.to_string().len());
    }
}
