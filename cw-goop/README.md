# CW-Goop

Cw-goop is a customized version of [Stargaze's Flexible Whitelist](https://github.com/public-awesome/launchpad/tree/main/contracts/whitelists/whitelist-flex).



## InstantiateMsg

```rust
#[cw_serde]
pub struct InstantiateMsg {
    pub members: Vec<Member>,
    pub claim_limit: u32,
    pub admins: Vec<String>,
    pub admins_mutable: bool,
}

```
json example:
```json
{
  "members": [
    {
      "address": "0x24EaSp0rts..",
      "headstash_amount": 123456789,
      "claim_count": 0
    },
    {
      "address": "0x23iMiNtHeGaMe...",
      "headstash_amount": 987654321,
      "claim_count": 1 
    }
  ],
  "claim_limit": 1,
  "admins": [
    "terp1...", 
    "terp1a...."
    ],
  "admins_mutable": true
}
```

# ExecuteMsg
```rs
#[cw_serde]
pub enum ExecuteMsg {
    AddMembers(AddMembersMsg),
    UpdateAdmins { admins: Vec<String> },
    Freeze {},
}
```

### AddMembers
```json
{
  "AddMembers": {
    "to_add": [
      {
        "address": "0x24EaSp0rts",
        "claim_count": 0
      },
      {
        "address": "0x23iMiNtHeGaMe",
        "claim_count": 0
      }
    ]
  }
}

```

### UpdateAdmins
```json
{
  "UpdateAdmins": {
    "admins": ["admin_address_1", "admin_address_2"]
  }
}

```

### Freeze
```json
{
  "Freeze": {}
}
```
# QueryMsg

### Members

### HasMember

### Member

### Config

### AdminList

### CanExecute

### PerAddressLimit