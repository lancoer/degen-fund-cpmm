pub mod instructions;

use anchor_lang::prelude::*;

use instructions::*;

#[cfg(feature = "devnet")]
declare_id!("Dw1WDYcV6RZDgwLfQY2wjzXZUTrbJSW5Cr1NFeNpPkKB");
#[cfg(not(feature = "devnet"))]
declare_id!("Dw1WDYcV6RZDgwLfQY2wjzXZUTrbJSW5Cr1NFeNpPkKB");

#[program]
pub mod degen_fund_cpmm {
    use super::*;

    /// Creates a pool for the given token pair and the initial price
    ///
    /// # Arguments
    ///
    /// * `ctx`- The context of accounts
    /// * `init_amount_0` - the initial amount_0 to deposit
    /// * `init_amount_1` - the initial amount_1 to deposit
    /// * `open_time` - the timestamp allowed for swap
    ///
    pub fn initialize(ctx: Context<Initialize>, init_amount_0: u64, init_amount_1: u64, open_time: u64) -> Result<()> {
        seed_spl_t22::seed_spl_t22(ctx, init_amount_0, init_amount_1, open_time)
    }
}
