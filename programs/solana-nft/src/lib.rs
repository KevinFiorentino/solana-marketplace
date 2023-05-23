use {
    anchor_lang::{
        prelude::*, solana_program::program::invoke, solana_program::program::invoke_signed,
        system_program, AnchorDeserialize, AnchorSerialize,
    },
    anchor_spl::{associated_token, token, token::Token},
    mpl_token_metadata::instruction::{
        approve_collection_authority, create_master_edition_v3, create_metadata_accounts_v3,
        set_and_verify_collection, sign_metadata, update_metadata_accounts_v2
    },
    mpl_token_metadata::state::Creator,
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
        Ok(())
    }

    pub fn mint_collection(
        ctx: Context<MintCollection>,
        collection_name: String,
        collection_symbol: String,
        metadata_uri: String,
        pic_url: String,
    ) -> Result<()> {

        // Create an account to become it in the token_mint 
        system_program::create_account(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                system_program::CreateAccount {
                    from: ctx.accounts.mint_authority.to_account_info(),
                    to: ctx.accounts.mint.to_account_info(),
                },
            ),
            1461600,
            82,
            &ctx.accounts.token_program.key(),
        )?;

        // Create the token_mint
        token::initialize_mint(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                token::InitializeMint {
                    mint: ctx.accounts.mint.to_account_info(),
                    rent: ctx.accounts.rent.to_account_info(),
                },
            ),
            0,
            &ctx.accounts.mint_authority.key(),
            Some(&ctx.accounts.mint_authority.key()),
        )?;

        // Create ATA for mint_authority (user_pda)
        associated_token::create(
            CpiContext::new(
                ctx.accounts.associated_token_program.to_account_info(),
                associated_token::Create {
                    payer: ctx.accounts.mint_authority.to_account_info(),
                    associated_token: ctx.accounts.token_account.to_account_info(),
                    authority: ctx.accounts.mint_authority.to_account_info(),
                    mint: ctx.accounts.mint.to_account_info(),
                    system_program: ctx.accounts.system_program.to_account_info(),
                    token_program: ctx.accounts.token_program.to_account_info(),
                },
            )
        )?;

        // Mint token
        token::mint_to(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                token::MintTo {
                    mint: ctx.accounts.mint.to_account_info(),
                    to: ctx.accounts.token_account.to_account_info(),
                    authority: ctx.accounts.mint_authority.to_account_info(),
                },
            ),
            1,
        )?;

        let creators = vec![
            mpl_token_metadata::state::Creator {
                address: ctx.accounts.mint_authority.key(),
                verified: false,
                share: 100,
            }
        ];

        // Create metadata for token_mint
        invoke(
            &create_metadata_accounts_v3(
                ctx.accounts.token_metadata_program.key(),
                ctx.accounts.metadata.key(),
                ctx.accounts.mint.key(),
                ctx.accounts.mint_authority.key(),
                ctx.accounts.payer.key(),
                ctx.accounts.payer.key(),
                collection_name.clone(),
                collection_symbol.clone(),
                metadata_uri,
                Some(creators),
                0,
                true,
                true,
                None,
                None,
                None,
            ),
            &[
                ctx.accounts.metadata.to_account_info(),
                ctx.accounts.mint.to_account_info(),
                ctx.accounts.mint_authority.to_account_info(),
                ctx.accounts.payer.to_account_info(),
                ctx.accounts.token_metadata_program.to_account_info(),
                ctx.accounts.token_program.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
                ctx.accounts.rent.to_account_info(),
            ],
        )?;

        // Sign metadata transaction
        invoke(
            &sign_metadata(
                ctx.accounts.token_metadata_program.key(),
                ctx.accounts.metadata.key(),
                ctx.accounts.mint_authority.key(),
            ),
            &[
                ctx.accounts.metadata.to_account_info(),
                ctx.accounts.mint_authority.to_account_info(),
            ],
        )?;

        // Create master edition
        invoke(
            &create_master_edition_v3(
                ctx.accounts.token_metadata_program.key(),
                ctx.accounts.master_edition.key(),
                ctx.accounts.mint.key(),
                ctx.accounts.payer.key(),
                ctx.accounts.mint_authority.key(),
                ctx.accounts.metadata.key(),
                ctx.accounts.payer.key(),
                Some(0),
            ),
            &[
                ctx.accounts.master_edition.to_account_info(),
                ctx.accounts.mint.to_account_info(),
                ctx.accounts.mint_authority.to_account_info(),
                ctx.accounts.payer.to_account_info(),
                ctx.accounts.metadata.to_account_info(),
                ctx.accounts.token_metadata_program.to_account_info(),
                ctx.accounts.token_program.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
                ctx.accounts.rent.to_account_info(),
            ],
        )?;

        invoke(
            &approve_collection_authority(
                ctx.accounts.token_metadata_program.key(),
                ctx.accounts.collection_authority_record.key(),
                ctx.accounts.collection_pda.key(),
                ctx.accounts.mint_authority.key(),
                ctx.accounts.mint_authority.key(),
                ctx.accounts.metadata.key(),
                ctx.accounts.mint.key(),
            ),
            &[
                ctx.accounts.collection_authority_record.to_account_info(),
                ctx.accounts.collection_pda.to_account_info(),
                ctx.accounts.mint_authority.to_account_info(),
                ctx.accounts.mint_authority.to_account_info(),
                ctx.accounts.metadata.to_account_info(),
                ctx.accounts.mint.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
                ctx.accounts.rent.to_account_info(),
            ]
        )?;

        invoke(
            &update_metadata_accounts_v2(
                ctx.accounts.token_metadata_program.key(),
                ctx.accounts.metadata.key(),
                ctx.accounts.payer.key(),
                Some(ctx.accounts.collection_pda.key()),
                None,
                Some(true),
                Some(false),
            ),
            &[
                ctx.accounts.token_metadata_program.to_account_info(),
                ctx.accounts.metadata.to_account_info(),
                ctx.accounts.payer.to_account_info(),
            ],
        )?;

        ctx.accounts.collection_pda.name = collection_name;
        ctx.accounts.collection_pda.pic_url = pic_url;
        ctx.accounts.collection_pda.symbol = collection_symbol;
        ctx.accounts.collection_pda.bump = *ctx.bumps.get("collection_pda").unwrap();
        ctx.accounts.collection_pda.collection_mint = ctx.accounts.mint.key();
        ctx.accounts.collection_pda.collection_id = ctx.accounts.user_pda.collections_qty;
        ctx.accounts.collection_pda.owner = ctx.accounts.payer.key();
        ctx.accounts.user_pda.collections_qty += 1;

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

#[derive(Accounts)]
pub struct MintCollection<'info> {
    /// CHECK:
    #[account(mut)]
    pub mint: Signer<'info>,

    /// CHECK:
    #[account(mut)]
    pub mint_authority: Signer<'info>,

    /// CHECK:
    #[account(mut)]
    pub payer: AccountInfo<'info>,

    pub rent: Sysvar<'info, Rent>,

    pub system_program: Program<'info, System>,

    #[account(
        mut,
        seeds = [
            b"user_account".as_ref(),
            payer.to_account_info().key.as_ref()
        ],
        bump = user_pda.bump
    )]
    user_pda: Account<'info, UserPDA>,

    pub token_program: Program<'info, token::Token>,

    /// CHECK:
    #[account(mut)]
    pub token_account: UncheckedAccount<'info>,

    pub associated_token_program: Program<'info, associated_token::AssociatedToken>,

    /// CHECK:
    pub token_metadata_program: UncheckedAccount<'info>,

    /// CHECK:
    #[account(mut)]
    pub master_edition: UncheckedAccount<'info>,

    /// CHECK:
    #[account(mut)]
    pub metadata: UncheckedAccount<'info>,

    /// CHECK:
    #[account(mut)]
    collection_authority_record: UncheckedAccount<'info>,

    #[account(
        init,
        payer=mint_authority,
        space = 1000,
        seeds = [
            b"collection".as_ref(),
            payer.to_account_info().key.as_ref(),
            user_pda.collections_qty.to_le_bytes().as_ref()
            // mint.to_account_info().key.as_ref()
        ],
        bump
    )]
    collection_pda: Box<Account<'info, CollectionPdaAccount>>,
}

#[account]
#[derive(Default)]
pub struct CollectionPdaAccount {
    pub pic_url: String,
    pub name: String,
    pub symbol: String,
    pub mint_number: u16,
    pub owner: Pubkey,
    pub collection_mint: Pubkey,
    pub collection_id: u64,
    pub bump: u8,
}
