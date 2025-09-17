use alloy::hex;
use cosmwasm_schema::cw_serde;
use cosmwasm_std::Api;
use cosmwasm_std::DepsMut;
use cosmwasm_std::Env;
use cosmwasm_std::instantiate2_address;
use cw20::Cw20ExecuteMsg;
use sha3::Digest;
use sha3::Keccak256;
use ucs03_zkgm::com::{
    Batch, Call, INSTR_VERSION_0, INSTR_VERSION_2, Instruction, OP_BATCH, OP_CALL, OP_TOKEN_ORDER,
    SolverMetadata, TOKEN_ORDER_KIND_SOLVE, TokenOrderV2,
};
use unionlabs_primitives::FixedBytes;
use unionlabs_primitives::encoding::HexPrefixed;

use crate::error::{ContractError, ContractResult};
use crate::helpers::query_contract_code_hash;
use crate::state::{LST_CONFIG, LstConfig, THIS_PROXY, TowerConfig, ZkgmLstConfig};
use alloy::sol_types::SolValue;
use alloy_primitives::Bytes as AlloyBytes;
use alloy_primitives::Uint;
use cosmwasm_std::{Addr, Coin, CosmosMsg, Uint64, Uint128, WasmMsg, to_json_binary};
use ibc_union_spec::{ChannelId, Duration, Timestamp};
use std::str::FromStr;
use ucs03_zkgm;
use unionlabs_primitives::{Bytes, H256, U256};

pub fn validate_and_store_zkgm_lst_config(
    deps: &mut DepsMut,
    config: &ZkgmLstConfig,
    tower_config: &TowerConfig,
) -> ContractResult<LstConfig> {
    validate_and_parse_channel_id(config.this_chain_channel_id)?;
    validate_and_parse_channel_id(config.lst_chain_channel_id)?;
    if tower_config.lp_underlying_asset.to_string() != config.underlying_base_token {
        return Err(ContractError::NonCompatibleZkgmLst {});
    }
    if tower_config.lp_other_asset.to_string() != config.lst_base_token {
        return Err(ContractError::NonCompatibleZkgmLst {});
    }
    let lst_config = LstConfig::Zkgm(config.clone());
    LST_CONFIG.save(deps.storage, &lst_config)?;
    Ok(lst_config)
}

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

// Bond request data
pub struct BondRequest {
    pub sender: String,        // this will be vault contract address
    pub proxy_account: String, // this will be proxy account contract address
    pub amount: Uint128,
    pub min_mint_amount: Uint128,
    pub denom: String,
}

// Config of zkgm and token for Vault on Babylon
pub struct VaultZkgmConfig {
    pub ucs03_zkgm: String,
    pub channel_id: u32,
    pub base_token: String,  // u contract address on babylon
    pub quote_token: String, // U contract address on union
}

// Config of zkgm and token related for LST Contract on Union
pub struct LstZkgmConfig {
    pub union_channel_id: u32,
    pub union_ucs03_zkgm: String,  // ucs03 zkgm contract address
    pub zkgm_token_minter: String, // zkgm token minter is the contract that handle minting and burn of eU
    pub lst_base_token: String,    // eU contract address on union
    pub lst_quote_token: String,   // eU contract address on babylon
    pub lst_contract_address: String,
    pub u_solver_address: String, // U solver address
}

type AlloyUint256 = Uint<256, 4>;

const TIMEOUT_OFFSET: u64 = 604_800; // 7 days period

/// Call LST bond function via ucs03 zkgm
///
/// # Errors
/// Will return error if messages fail to serialize or validation fails
pub fn call_lst_bond(
    bond_request: &BondRequest,
    vault_zkgm_config: VaultZkgmConfig,
    lst_zkgm_config: LstZkgmConfig,
    time: Timestamp,
    salt: &str,
) -> ContractResult<CosmosMsg> {
    let proxy_account_address = validate_and_parse_hex(bond_request.proxy_account.as_ref())?;
    let quote_token = validate_and_parse_hex(vault_zkgm_config.quote_token.as_ref())?;

    let sender_bytes: AlloyBytes = bond_request.sender.as_bytes().to_vec().into();

    let metadata = SolverMetadata {
        solverAddress: Vec::from(lst_zkgm_config.u_solver_address.clone()).into(),
        metadata: AlloyBytes::default(),
    };

    // Create token order v2 to send U from babylon vault to union LST
    let token_order_v2 = Instruction {
        version: INSTR_VERSION_2,
        opcode: OP_TOKEN_ORDER,
        operand: TokenOrderV2 {
            sender: sender_bytes.clone(), // vault is the sender
            receiver: Vec::from(proxy_account_address.clone()).into(),
            base_token: vault_zkgm_config.base_token.as_bytes().to_vec().into(),
            base_amount: AlloyUint256::from(bond_request.amount.u128()),
            quote_token: Vec::from(quote_token).into(),
            quote_amount: AlloyUint256::from(bond_request.amount.u128()),
            kind: TOKEN_ORDER_KIND_SOLVE,
            metadata: metadata.abi_encode_params().into(),
        }
        .abi_encode_params()
        .into(),
    };

    let timeout_timestamp = get_timeout_timestamp_from_time(time)?;

    let salt = validate_and_parse_salt(salt)?;

    // Generate 3 payloads as array/vector: bond, increase_allowance and send back via tokenorderv2, these need to be encoded as Bytes
    let contract_calldata =
        generate_bond_calldata(bond_request, lst_zkgm_config, timeout_timestamp, salt)?;

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
        operand: Batch { instructions: vec![token_order_v2, call_instruction] }
            .abi_encode_params()
            .into(),
    };

    let channel_id = validate_and_parse_channel_id(vault_zkgm_config.channel_id)?;

    let ucs03_send_msg = ucs03_zkgm::msg::ExecuteMsg::Send {
        channel_id,
        timeout_height: Uint64::from(0u64),
        timeout_timestamp,
        salt,
        instruction: batch_instruction.abi_encode_params().into(),
    };

    let send_msg = to_json_binary(&ucs03_send_msg)?;
    let execute_increase_allowance_msg: CosmosMsg = WasmMsg::Execute {
        contract_addr: vault_zkgm_config.ucs03_zkgm,
        msg: send_msg,
        funds: vec![],
    }
    .into();

    Ok(execute_increase_allowance_msg)
}

/// # Errors
/// Will return error if overflow
pub fn get_timeout_timestamp_from_time(time: Timestamp) -> ContractResult<Timestamp> {
    let duration_offset = Duration::from_secs(TIMEOUT_OFFSET);
    Ok(Timestamp::from_nanos(
        time.plus_duration(duration_offset).ok_or(ContractError::TimestampOverflow {})?.as_nanos(),
    ))
}

/// Validates and returns `channel_id` as `ChannelId` for usage with ZKGM
///
/// # Errors
/// Will return error if validations fail
pub fn validate_and_parse_channel_id(channel_id: u32) -> ContractResult<ChannelId> {
    ChannelId::from_raw(channel_id).ok_or(ContractError::InvalidChannelId {})
}

/// Validates and returns `salt` as `unionlabs_primitives::H256` for usage with ZKGM
///
/// # Errors
/// Will return error if validations fail
pub fn validate_and_parse_salt(salt: &str) -> ContractResult<unionlabs_primitives::H256> {
    unionlabs_primitives::H256::from_str(salt).map_err(|_| ContractError::InvalidSalt {})
}

/// Validates and returns `address` as `Bytes` for usage with ZKGM
///
/// # Errors
/// Will return error if validations fail
pub fn validate_and_parse_hex(address: &str) -> ContractResult<Bytes> {
    let hex_address = if address.starts_with("0x") {
        address.to_string()
    } else {
        format!("0x{}", hex::encode(address.as_bytes()))
    };
    Bytes::from_str(&hex_address).map_err(|_| ContractError::InvalidHex {})
}

#[cw_serde]
pub enum TestCW20ExecuteMsg {
    IncreaseAllowance { spender: String, amount: Uint128 },
}

/// # Errors
/// Will return error if validations fail
pub fn generate_bond_calldata(
    request: &BondRequest,
    lst_zkgm_config: LstZkgmConfig,
    timeout: Timestamp,
    salt: unionlabs_primitives::H256,
) -> ContractResult<Bytes> {
    let lst_contract_address = lst_zkgm_config.lst_contract_address; // liquid staking contract address
    let lst_base_token = lst_zkgm_config.lst_base_token; // eU contract address on union
    let lst_quote_token = lst_zkgm_config.lst_quote_token; // eU contract address on babylon
    let union_ucs03_zkgm = lst_zkgm_config.union_ucs03_zkgm;
    let zkgm_token_minter = lst_zkgm_config.zkgm_token_minter;

    let mut call_msgs: Vec<CosmosMsg> = vec![];

    // 1. construct bond msg call to LST

    let bond_msg = LstExecuteMsg::Bond {
        mint_to_address: Addr::unchecked(request.proxy_account.clone()),
        min_mint_amount: request.min_mint_amount,
    };
    let execute_bond_msg = WasmMsg::Execute {
        contract_addr: lst_contract_address.clone(),
        msg: to_json_binary(&bond_msg)?,
        funds: vec![Coin { denom: request.denom.clone(), amount: request.amount }],
    };
    call_msgs.push(execute_bond_msg.into());

    // 2. construct increase allowance to zkgm token minter

    let increase_allowance_msg = Cw20ExecuteMsg::IncreaseAllowance {
        spender: zkgm_token_minter.clone(),
        amount: if cfg!(test) { request.amount } else { request.min_mint_amount },
        expires: None,
    };

    let execute_increase_allowance_msg = WasmMsg::Execute {
        contract_addr: lst_base_token.clone(), // it is eU contract adddress on Union
        msg: to_json_binary(&increase_allowance_msg)?,
        funds: vec![],
    };
    call_msgs.push(execute_increase_allowance_msg.into());

    let quote_token =
        validate_and_parse_hex(if cfg!(test) { &lst_quote_token } else { &request.proxy_account })?;

    let metadata = SolverMetadata {
        solverAddress: quote_token.clone().into(),
        metadata: alloy_primitives::Bytes::default(),
    };

    // 3. construct token order v2 to send back from to sender

    let sender_bytes = validate_and_parse_hex(if cfg!(test) {
        "union1ylfrhs2y5zdj2394m6fxgpzrjav7le3z07jffq"
    } else {
        &request.proxy_account
    })?;

    let receiver = validate_and_parse_hex(&request.sender)?.into();

    let fungible_order_instruction = Instruction {
        version: INSTR_VERSION_2,
        opcode: OP_TOKEN_ORDER,
        operand: TokenOrderV2 {
            sender: sender_bytes.into(),
            receiver,
            base_token: Vec::from(lst_base_token).into(), // eU contract address on union
            base_amount: AlloyUint256::from(request.min_mint_amount.u128()),
            quote_token: quote_token.into(), // eU contract address on babylon
            quote_amount: AlloyUint256::from(request.min_mint_amount.u128()),
            kind: TOKEN_ORDER_KIND_SOLVE,
            metadata: metadata.abi_encode_params().into(),
        }
        .abi_encode_params()
        .into(),
    };
    let send_msg = ucs03_zkgm::msg::ExecuteMsg::Send {
        channel_id: validate_and_parse_channel_id(lst_zkgm_config.union_channel_id)?,
        timeout_height: Uint64::from(0u64),
        timeout_timestamp: timeout,
        salt,
        instruction: fungible_order_instruction.abi_encode_params().into(),
    };

    let execute_send_msg = WasmMsg::Execute {
        contract_addr: union_ucs03_zkgm, // UCS03 contract on Union
        msg: to_json_binary(&send_msg)?,
        funds: vec![],
    };

    call_msgs.push(execute_send_msg.into());

    let msgs_bin = to_json_binary(&call_msgs)?;
    let msgs_bytes: Bytes = msgs_bin.to_vec().into();
    Ok(msgs_bytes)
}

pub fn predict_proxy_account(
    api: &dyn Api,
    path: U256,
    channel_id: ChannelId,
    user: &Bytes,
    code_hash: &str,
    creator_addr: &str,
) -> Result<Addr, ContractError> {
    let salt: H256 = Keccak256::new()
        .chain_update(
            (Into::<alloy_primitives::U256>::into(path), channel_id.raw(), user.clone())
                .abi_encode_params(),
        )
        .finalize()
        .into();
    let code_hash: FixedBytes<32, HexPrefixed> =
        H256::from_str(code_hash).map_err(|_| ContractError::InvalidHex {})?;
    let token_addr = instantiate2_address(
        &code_hash.into_bytes(),
        &api.addr_canonicalize(creator_addr)?,
        salt.get().as_slice(),
    )?;
    Ok(api.addr_humanize(&token_addr)?)
}

pub fn update_this_proxy(
    deps: &mut DepsMut,
    env: &Env,
    config: &ZkgmLstConfig,
) -> ContractResult<Addr> {
    let this = validate_and_parse_hex(env.contract.address.as_ref())?;
    let contract_addr = config.this_chain_ucs03_zkgm.clone();
    let code_hash =
        query_contract_code_hash(&deps.as_ref(), deps.api.addr_validate(&contract_addr)?)?;
    let proxy = predict_proxy_account(
        deps.api,
        U256::from(0_u32),
        validate_and_parse_channel_id(config.this_chain_channel_id)?,
        &this,
        &code_hash,
        &contract_addr,
    )?;
    THIS_PROXY.save(deps.storage, &proxy)?;
    Ok(proxy)
}

pub fn get_or_update_this_proxy(
    deps: &mut DepsMut,
    env: &Env,
    config: &ZkgmLstConfig,
) -> ContractResult<Addr> {
    Ok(THIS_PROXY.may_load(deps.storage)?.unwrap_or(update_this_proxy(deps, env, config)?))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::zkgm::generate_bond_calldata;
    use cosmwasm_std::HexBinary;
    use cosmwasm_std::Uint128;
    use cosmwasm_std::from_json;
    use cw_multi_test::AppBuilder;
    use cw_multi_test::MockApiBech32;
    use cw20::Cw20ExecuteMsg;
    use ibc_union_spec::ChannelId;
    use ibc_union_spec::Timestamp;
    use std::str::FromStr;

    #[test]
    /// This test will try to match contract calldata of Call from this packet below:
    /// https://app.union.build/explorer/packets/0x270a1a682466bd78717b4170ff3575580cda0af1a98a8fa0b10f710f0f96974d
    fn test_generate_bond_calldata() {
        let amount = Uint128::from(1_000_000_000_000_000_000u128);
        let sender = "0x4aAa51a0814D91F7D2B3Ab60829A921eC9Eb8e17".to_string();
        let denom = "au".to_string();
        let min_mint_amount = Uint128::from(900_000_000_000_000_000u128);

        let proxy_account =
            "union1cnntu09j3s49kqspr7st5rpatvn68h4yueapjskc9sug8t2mu7ys46tfq9".to_string();
        let timeout = Timestamp::from_nanos(1_757_853_925_422_000_000);

        let salt: unionlabs_primitives::H256 = unionlabs_primitives::H256::from_str(
            "0x64956543467d00f9180e2a2ca78ccec3ef63c7f7490a62b86594fe14b21225fb",
        )
        .unwrap();

        let zkgm_token_minter =
            "union1t5awl707x54k6yyx7qfkuqp890dss2pqgwxh07cu44x5lrlvt4rs8hqmk0".to_string();
        let lst_contract_address =
            "union1d2r4ecsuap4pujrlf3nz09vz8eha8y0z25knq0lfxz4yzn83v6kq0jxsmk".to_string();
        let lst_base_token =
            "union1eueueueu9var4yhdruyzkjcsh74xzeug6ckyy60hs0vcqnzql2hq0lxc2f".to_string();
        let lst_quote_token = "0xe5Cf13C84c0fEa3236C101Bd7d743d30366E5CF1".to_string();
        let union_ucs03_zkgm =
            "union1336jj8ertl8h7rdvnz4dh5rqahd09cy0x43guhsxx6xyrztx292qpe64fh".to_string();
        let u_solver_address =
            "union1uuuuuuuuu9un2qpksam7rlttpxc8dc76mcphhsmp39pxjnsvrtcqvyv57r".to_string();

        let result = generate_bond_calldata(
            &BondRequest { sender, proxy_account, amount, min_mint_amount, denom },
            LstZkgmConfig {
                union_ucs03_zkgm,
                zkgm_token_minter,
                lst_base_token,
                lst_quote_token,
                lst_contract_address,
                union_channel_id: 20,
                u_solver_address,
            },
            timeout,
            salt,
        )
        .unwrap();

        let expected_output = "0x5b7b227761736d223a7b2265786563757465223a7b22636f6e74726163745f61646472223a22756e696f6e31643272346563737561703470756a726c66336e7a3039767a386568613879307a32356b6e71306c66787a34797a6e383376366b71306a78736d6b222c226d7367223a2265794a696232356b496a7037496d3170626e5266644739665957526b636d567a63794936496e5675615739754d574e75626e52314d446c714d334d304f577478633342794e334e304e584a7759585232626a593461445235645756686347707a61324d356333566e4f4851796258553365584d304e6e526d63546b694c434a746157356662576c756446396862573931626e51694f6949354d4441774d4441774d4441774d4441774d4441774d4441696658303d222c2266756e6473223a5b7b2264656e6f6d223a226175222c22616d6f756e74223a2231303030303030303030303030303030303030227d5d7d7d7d2c7b227761736d223a7b2265786563757465223a7b22636f6e74726163745f61646472223a22756e696f6e31657565756575657539766172347968647275797a6b6a6373683734787a65756736636b797936306873307663716e7a716c326871306c78633266222c226d7367223a2265794a70626d4e795a57467a5a5639686247787664324675593255694f6e73696333426c626d526c63694936496e5675615739754d585131595864734e7a413365445530617a5a356558673363575a72645846774f446b775a484e7a4d6e42785a33643461444133593355304e48673162484a73646e5130636e4d3461484674617a41694c434a6862573931626e51694f6949784d4441774d4441774d4441774d4441774d4441774d444177496e3139222c2266756e6473223a5b5d7d7d7d2c7b227761736d223a7b2265786563757465223a7b22636f6e74726163745f61646472223a22756e696f6e313333366a6a386572746c3868377264766e7a3464683572716168643039637930783433677568737878367879727a747832393271706536346668222c226d7367223a2265794a7a5a57356b496a7037496d4e6f595735755a577866615751694f6a49774c434a306157316c623356305832686c6157646f64434936496a41694c434a306157316c62335630583352706257567a6447467463434936496a45334e5463344e544d354d6a55304d6a49774d4441774d4441694c434a7a59577830496a6f694d4867324e446b314e6a55304d7a51324e3251774d4759354d5467775a544a684d6d4e684e7a686a5932566a4d32566d4e6a4e6a4e3259334e446b7759545979596a67324e546b305a6d55784e4749794d5449794e575a69496977696157357a64484a3159335270623234694f694977654441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4449774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d44417a4d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441324d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d44417a4d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d5441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4445324d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d444178595441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d444177597a646b4e7a457a596a51355a4745774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4449774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d44426a4e3251334d544e694e446c6b595441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d44417a4d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4449304d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d6d4d334e545a6c4e6a6b325a6a5a6c4d7a45334f545a6a4e6a59334d6a59344e7a4d7a4d6a63354d7a5533595459304e6d457a4d6a4d7a4d7a6b7a4e445a6b4d7a59324e6a63344e6a63334d4464684e7a4932595459784e7a597a4e7a5a6a4e6a557a4d7a64684d7a417a4e7a5a684e6a59324e6a63784d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d545130595546684e5446684d4467784e4551354d55593352444a434d3046694e6a41344d6a6c424f5449785a554d35525749345a5445334d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441304d4463314e6d55324f545a6d4e6d557a4d5459314e7a55324e5463314e6a55334e5459314e7a557a4f5463324e6a45334d6a4d304e7a6b324f4459304e7a49334e5463354e324532596a5a684e6a4d334d7a59344d7a637a4e4463344e3245324e5463314e6a637a4e6a597a4e6d49334f5463354d7a597a4d4459344e7a4d7a4d4463324e6a4d334d545a6c4e3245334d545a6a4d7a49324f4463784d7a4132597a63344e6a4d7a4d6a59324d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441784e475531593259784d324d344e474d775a6d56684d7a497a4e6d4d784d4446695a44646b4e7a517a5a444d774d7a59325a54566a5a6a45774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4745774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441304d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774f4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4445305a54566a5a6a457a597a6730597a426d5a57457a4d6a4d32597a45774d574a6b4e3251334e444e6b4d7a417a4e6a5a6c4e574e6d4d5441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441774d4441696658303d222c2266756e6473223a5b5d7d7d7d5d";
        let expected_msgs: Vec<CosmosMsg> =
            from_json(HexBinary::from_hex(expected_output.strip_prefix("0x").unwrap()).unwrap())
                .unwrap();
        let result_msgs: Vec<CosmosMsg> =
            from_json(HexBinary::from_hex(result.to_string().strip_prefix("0x").unwrap()).unwrap())
                .unwrap();
        assert_eq!(result_msgs.len(), expected_msgs.len());
        for i in 0..result_msgs.len() {
            let result_msg = &result_msgs[i];
            let expected_msg = &expected_msgs[i];
            let CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: result_contract_addr,
                msg: result_msg,
                funds: result_funds,
            }) = result_msg
            else {
                panic!()
            };
            let CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: expected_contract_addr,
                msg: expected_msg,
                funds: expected_funds,
            }) = expected_msg
            else {
                panic!()
            };
            assert_eq!(result_contract_addr, expected_contract_addr);
            assert_eq!(result_funds, expected_funds);

            // Assert that the data is the same
            match i {
                0 => {
                    let LstExecuteMsg::Bond {
                        mint_to_address: result_mint_to_address,
                        min_mint_amount: result_min_mint_amount,
                    } = from_json(result_msg).unwrap()
                    else {
                        panic!()
                    };
                    let LstExecuteMsg::Bond {
                        mint_to_address: expected_mint_to_address,
                        min_mint_amount: expected_min_mint_amount,
                    } = from_json(expected_msg).unwrap()
                    else {
                        panic!()
                    };
                    assert_eq!(result_mint_to_address, expected_mint_to_address);
                    assert_eq!(result_min_mint_amount, expected_min_mint_amount);
                }
                1 => {
                    let Cw20ExecuteMsg::IncreaseAllowance {
                        spender: result_spender,
                        amount: result_amount,
                        expires: result_expires,
                    } = from_json(result_msg).unwrap()
                    else {
                        panic!()
                    };
                    let Cw20ExecuteMsg::IncreaseAllowance {
                        spender: expected_spender,
                        amount: expected_amount,
                        expires: expected_expires,
                    } = from_json(expected_msg).unwrap()
                    else {
                        panic!()
                    };
                    assert_eq!(result_spender, expected_spender);
                    assert_eq!(result_amount, expected_amount);
                    assert_eq!(result_expires, expected_expires);
                }
                2 => {
                    let ucs03_zkgm::msg::ExecuteMsg::Send {
                        channel_id: result_channel_id,
                        timeout_height: result_timeout_height,
                        timeout_timestamp: result_timeout_timestamp,
                        salt: result_salt,
                        instruction: result_instruction,
                    } = from_json(result_msg).unwrap()
                    else {
                        panic!()
                    };
                    let ucs03_zkgm::msg::ExecuteMsg::Send {
                        channel_id: expected_channel_id,
                        timeout_height: expected_timeout_height,
                        timeout_timestamp: expected_timeout_timestamp,
                        salt: expected_salt,
                        instruction: expected_instruction,
                    } = from_json(expected_msg).unwrap()
                    else {
                        panic!()
                    };
                    assert_eq!(result_channel_id, expected_channel_id);
                    assert_eq!(result_timeout_height, expected_timeout_height);
                    assert_eq!(result_timeout_timestamp, expected_timeout_timestamp);
                    assert_eq!(result_salt, expected_salt);
                    assert_eq!(result_instruction, expected_instruction);
                }
                _ => {
                    panic!()
                }
            }
        }
    }

    #[test]
    fn instantiate2_address() {
        let addr = "0x4aaa51a0814d91f7d2b3ab60829a921ec9eb8e17";
        let expected = "union1cnntu09j3s49kqspr7st5rpatvn68h4yueapjskc9sug8t2mu7ys46tfq9";
        let union_ucs03_zkgm = "union1336jj8ertl8h7rdvnz4dh5rqahd09cy0x43guhsxx6xyrztx292qpe64fh";
        let code_hash = "0xec827349ed4c1fec5a9c3462ff7c979d4c40e7aa43b16ed34469d04ff835f2a1";

        let app = AppBuilder::default().with_api(MockApiBech32::new("union")).build(|_, _, _| {});
        let api = app.api();
        let predicted = predict_proxy_account(
            api,
            U256::from(0_u32),
            ChannelId::from_raw(20).unwrap(),
            &validate_and_parse_hex(addr).unwrap(),
            code_hash,
            union_ucs03_zkgm,
        )
        .unwrap();

        assert_eq!(predicted.to_string(), expected);
    }
}
