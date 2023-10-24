# Headstash Contract

These contracts handle how headstash recipients can verify ownership & claim their allocations.

## contract checks
### `validate_claim` 
- validates provided `eth_address` is an eligible headstash recipient
- `validate_eth_sig`  ensures the eth signature string provided is derived from the pubkey of the eth address provided
- `validate_claims_remaining` ensures that the `eth_address` being claimed has not already been claimed.

## to-do
- create `load_goop_member()`
- create `query_headstash_goop()`

## InstantiateMsg
```rs
#[cw_serde]
pub struct InstantiateMsg {
    pub admin: Vec<String>,
    pub claim_msg_plaintext: String, // {wallet}
    pub members: Vec<Member>, 
    pub cw_goop_id: u64, 
    pub claim_limit: u32, // 1 claim per address
    pub admins_mutable: bool,
}
```

## ExecuteMsg
### ClaimHeadstash
```rs
ClaimHeadstash {
        eth_address: String,
        eth_sig: String,
    },
```
json example
```json
{
  "admin": ["terp1...", "terp1a..."],
  "claim_msg_plaintext": "{wallet}",
  "members": [
    {
      "address": "0x24EaSports",
      "mint_count": 0
    },
    {
      "address": "0x23Imin",
      "mint_count": 0
    }
  ],
  "cw_goop_id": 123,
  "claim_limit": 10,
  "admins_mutable": true
}
```

## Query
```rs
#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(bool)]
    HeadstashEligible { eth_address: String },
}

```
### HeadstashEligible - boolean
```rs
HeadstashEligible { eth_address: String }
```
json example: 
```json
{
  "HeadstashEligible": {
    "eth_address": "your_ethereum_address"
  }
}
```

### Useful Notes

#### compute_plaintext_msg
Located in [src/claim_headstash.rs](./src/claim_headstash.rs), this function is used to populate the `{wallet}` variable from a ui client, into the message that is being signed on the client. 
```
 pub fn compute_plaintext_msg(config: &Config, info: MessageInfo) -> String {
        str::replace(
            &config.claim_msg_plaintext,
            "{wallet}",
            info.sender.as_ref(),
        )
    }
```
