use crate::*;
use pyth_sdk_solana::state::load_price_account;
use std::ops::Deref;
use std::str::FromStr;

use crate::PythErrorCode;



#[account]
pub struct Pricefeedaccount {
    // Only have SOL/USD price feed for now
    pub pricefeed_alias: String,
    pub pricefeed_id: Pubkey,
    pub bump: u8,
}


#[account]
pub struct Locker {
    pub authority: Pubkey,
    pub creation_ts: u32,
    pub locked_balance: u32,
    pub locks: Vec<Lock>,
    pub locked: bool,
    pub staked: bool,
    pub bump: u8,
}

#[derive(Debug, AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq)]
// added the initspace macro here against ChatGPT's advice, but seems necessary
// look here if the account structure is throwing errors
pub enum Lock {
    TimeLock {
        id: u8,
        strike_time: u32, 
        amount: u32,
        token_mint: Option<Pubkey>,
        locked: bool,
        join: Option<u8>,
    },
    PriceLock {
        id: u8,
        strike_price: u32,
        amount: u32, 
        token_mint: Option<Pubkey>,
        locked: bool,
        join: Option<u8>,
    },
}



// PYTH integrations


#[derive(Clone)]
pub struct PriceFeed(pyth_sdk::PriceFeed);

impl anchor_lang::Owner for PriceFeed {
    fn owner() -> Pubkey {
        // Make sure the owner is the pyth oracle account on solana devnet
        let oracle_addr = "gSbePebfvPy7tRqimPoVecS2UsBvYv46ynrzWocc92s";
        return Pubkey::from_str(&oracle_addr).unwrap();
    }
}

impl anchor_lang::AccountDeserialize for PriceFeed {
    fn try_deserialize_unchecked(data: &mut &[u8]) -> Result<Self> {
        let account = load_price_account(data).map_err(|_x| error!(PythErrorCode::PythError))?;

        // Use a dummy key since the key field will be removed from the SDK
        let zeros: [u8; 32] = [0; 32];
        let dummy_key = Pubkey::from(zeros);
        let feed = account.to_price_feed(&dummy_key);
        return Ok(PriceFeed(feed));
    }
}

impl anchor_lang::AccountSerialize for PriceFeed {
    fn try_serialize<W: std::io::Write>(&self, _writer: &mut W) -> std::result::Result<(), Error> {
        Err(error!(PythErrorCode::TryToSerializePriceAccount))
    }
}

impl Deref for PriceFeed {
    type Target = pyth_sdk::PriceFeed;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}



