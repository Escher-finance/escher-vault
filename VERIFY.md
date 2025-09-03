# Deployment Code Verification

Using our current setup it is possible to verify which commit corresponds to a
certain deployment of `cw4626-escher`.

1. Getting Git info from the deployed contract
   - Query the deployed contract with `{ "git_info": {} }`
   - That should return a string in the format `{git branch}:{git commit hash}`
2. Checkout to that git branch and commit
3. Build the optimized code
   - `$ nix develop`
   - `$ ./scripts/build-optimize.sh`
   - You should now have `./artifacts/cw4626_escher.wasm`
4. Compute the hash of that file
   - `$ sha256sum ./artifacts/cw4626_escher.wasm`
   - Write down that hash named as `A_HASH`
5. Query the deployed contract code hash
   - Get the code-id
     `$ babylond query wasm contract {contract_addr} | jq .contract_info.code_id`
   - `$ babylond query wasm code-info {code_id} | jq .checksum`
   - Write down that hash named as `B_HASH`
6. Verify that `A_HASH` = `B_HASH` ✅
