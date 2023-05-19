use {
    anchor_lang::{
        prelude::*/* , solana_program::program::invoke, solana_program::program::invoke_signed,
        system_program, AnchorDeserialize, AnchorSerialize, */
    },
    /* anchor_spl::{associated_token, token, token::Token},
    mpl_token_metadata::instruction::{
        approve_collection_authority, create_master_edition_v3, create_metadata_accounts_v3,
        set_and_verify_collection, sign_metadata, update_metadata_accounts_v2,
    },
    mpl_token_metadata::state::Creator, */
};

declare_id!("GjsR1GVT5G51oMuTDrRzPporaknzWM39TgJs9n84Wmti");

#[program]
pub mod solana_nft {
    use super::*;

    pub fn create_user_account(
        ctx: Context<CreateUserAccount>
    ) -> Result<()> {
        ctx.accounts.user_pda.bump = *ctx.bumps.get("user_pda").unwrap();
        ctx.accounts.user_pda.collections_qty = 0;

        msg!("user_pda: {}", *ctx.bumps.get("user_pda").unwrap());

        Ok(())
    }

}

#[derive(Accounts)]
pub struct CreateUserAccount<'info> {
    #[account(
        init,
        payer = payer,
        space = 200,
        seeds = [
            b"user_account".as_ref(), payer.to_account_info().key.as_ref()
        ],
        bump
    )]
    user_pda: Account<'info, UserPDA>,
    #[account(mut)]
    pub payer: Signer<'info>,
    pub rent: Sysvar<'info, Rent>,
    pub system_program: Program<'info, System>,
}

#[account]
#[derive(Default)]
pub struct UserPDA {
    pub collections_qty: u64,
    pub bump: u8,
}
