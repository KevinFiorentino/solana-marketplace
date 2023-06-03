use {
    anchor_lang::{
        prelude::*, system_program, solana_program::program::invoke, solana_program::program::invoke_signed,
    },
    anchor_spl::{token, token::Token, associated_token},
    mpl_token_metadata::instruction::{
        create_master_edition_v3, create_metadata_accounts_v3, update_metadata_accounts_v2,
        approve_collection_authority, set_and_verify_collection, sign_metadata, 
    }
};

const DISCRIMINATOR_LENGTH: usize = 8;
const PUBLIC_KEY_LENGTH: usize = 32;
const STRING_PREFIX_LENGTH: usize = 4;
const I64_LENGTH: usize = 8;
const U8_LENGTH: usize = 1;

declare_id!("CsvQG7uR6AzA53YiHwkyfGSvpd54JdjdFgEDvU5WSXBb");

#[program]
pub mod solana_nft {
    use super::*;

    pub fn mint_collection(
        ctx: Context<MintCollection>,
        collection_name: String,
        collection_symbol: String,
        ipfs_image_hash: String,
        metadata_uri: String,
    ) -> Result<()> {

        // Create an account to become it in the collection token_mint 
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

        // Create the token_mint for the collection
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

        // Create ATA for mint_authority
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

        // Mint collection token
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

        // Create metadata for the collection token_mint
        let creators = vec![
            mpl_token_metadata::state::Creator {
                address: ctx.accounts.mint_authority.key(),
                verified: false,
                share: 100,
            }
        ];

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

        // Create master edition for the collection
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

        // Change collection authority to collection_pda
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

        // Change update authority to collection_pda
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

        // Set custom data collection account
        let clock: Clock = Clock::get().unwrap();

        ctx.accounts.collection_pda.owner = ctx.accounts.payer.key();
        ctx.accounts.collection_pda.token_mint = ctx.accounts.mint.key();
        ctx.accounts.collection_pda.name = collection_name;
        ctx.accounts.collection_pda.symbol = collection_symbol;
        ctx.accounts.collection_pda.ipfs_image_hash = ipfs_image_hash;
        ctx.accounts.collection_pda.bump = *ctx.bumps.get("collection_pda").unwrap();
        ctx.accounts.collection_pda.created = clock.unix_timestamp;

        Ok(())
    }

    pub fn mint_nft_from_collection(
        ctx: Context<MintNftFromCollection>,
        nft_name: String,
        ipfs_image_hash: String,
        metadata_uri: String,
    ) -> Result<()> {

        // Create an account to become it in the NFT token_mint
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

        // Create the token_mint for the NFT
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

        // Create ATA for mint_authority
        associated_token::create(CpiContext::new(
            ctx.accounts.associated_token_program.to_account_info(),
            associated_token::Create {
                payer: ctx.accounts.mint_authority.to_account_info(),
                associated_token: ctx.accounts.token_account.to_account_info(),
                authority: ctx.accounts.mint_authority.to_account_info(),
                mint: ctx.accounts.mint.to_account_info(),
                system_program: ctx.accounts.system_program.to_account_info(),
                token_program: ctx.accounts.token_program.to_account_info(),
            },
        ))?;

        // Mint NFT
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

        // Create metadata for the NFT token_mint
        invoke(
            &create_metadata_accounts_v3(
                ctx.accounts.token_metadata_program.key(),
                ctx.accounts.metadata.key(),
                ctx.accounts.mint.key(),
                ctx.accounts.mint_authority.key(),
                ctx.accounts.payer.key(),
                ctx.accounts.payer.key(),
                nft_name.clone(),
                ctx.accounts.collection_pda.symbol.to_string(),
                metadata_uri.clone(),
                None,
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

        // Create master edition for the NFT
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

        // Change update authority to nft_pda and set metadata
        let creators = vec![
            mpl_token_metadata::state::Creator {
                address: ctx.accounts.mint_authority.key(),
                verified: false,
                share: 100,
            }
        ];

        let data = mpl_token_metadata::state::DataV2 {
            name: nft_name.clone(),
            symbol: ctx.accounts.collection_pda.symbol.clone(),
            uri: metadata_uri.clone(),
            collection: None,
            creators: Some(creators),
            seller_fee_basis_points: 0,
            uses: None,
        };

        invoke(
            &update_metadata_accounts_v2(
                ctx.accounts.token_metadata_program.key(),
                ctx.accounts.metadata.key(),
                ctx.accounts.payer.key(),
                Some(ctx.accounts.collection_pda.key()),
                Some(data),
                Some(true),
                Some(true),
            ),
            &[
                ctx.accounts.token_metadata_program.to_account_info(),
                ctx.accounts.metadata.to_account_info(),
                ctx.accounts.payer.to_account_info(),
            ],
        )?;

        // Sign Metadata (verify creator)
        let coll_mint = ctx.accounts.collection_pda.token_mint;
        let coll_bump = ctx.accounts.collection_pda.bump;
        let _signer_seeds = [
            b"collection".as_ref(),
            coll_mint.as_ref(),
            &[coll_bump],
        ];

        invoke_signed(
            &sign_metadata(
                ctx.accounts.token_metadata_program.key(),
                ctx.accounts.metadata.key(),
                ctx.accounts.mint_authority.key(),
            ),
            &[
                ctx.accounts.metadata.to_account_info(),
                ctx.accounts.mint_authority.to_account_info(),
            ],
            &[&_signer_seeds],
        )?;

        // Verify master edition
        invoke_signed(
            &set_and_verify_collection(
                ctx.accounts.token_metadata_program.key(),
                ctx.accounts.metadata.key(),
                ctx.accounts.collection_pda.key(),
                ctx.accounts.payer.key(),
                ctx.accounts.collection_pda.key(),
                ctx.accounts.collection_token_mint.key(),
                ctx.accounts.collection_metadata.key(),
                ctx.accounts.collection_master_ed.key(),
                None,
            ),
            &[
                ctx.accounts.metadata.to_account_info(),
                ctx.accounts.collection_pda.to_account_info(),
                ctx.accounts.payer.to_account_info(),
                ctx.accounts.collection_pda.to_account_info(),
                ctx.accounts.collection_token_mint.to_account_info(),
                ctx.accounts.collection_metadata.to_account_info(),
                ctx.accounts.collection_master_ed.to_account_info(),
            ],
            &[&_signer_seeds],
        )?;

        // Set NFT data
        let clock: Clock = Clock::get().unwrap();

        ctx.accounts.nft_pda.token_mint = ctx.accounts.mint.key();
        ctx.accounts.nft_pda.collection_mint = ctx.accounts.collection_token_mint.key();
        ctx.accounts.nft_pda.name = nft_name;
        ctx.accounts.nft_pda.ipfs_image_hash = ipfs_image_hash;
        ctx.accounts.nft_pda.created = clock.unix_timestamp;

        Ok(())
    }

}

#[derive(Accounts)]
#[instruction(
    collection_name: String,
    collection_symbol: String,
    ipfs_image_hash: String,
    _metadata_uri: String,
)]
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
        payer = mint_authority,
        space = CollectionAccount::get_space(
            collection_name,
            collection_symbol,
            ipfs_image_hash
        ),
        seeds = [
            b"collection".as_ref(),
            mint.to_account_info().key.as_ref()
        ],
        bump
    )]
    collection_pda: Box<Account<'info, CollectionAccount>>,
}

#[derive(Accounts)]
#[instruction(
    nft_name: String,
    ipfs_image_hash: String,
    _metadata_uri: String,
)]
pub struct MintNftFromCollection<'info> {
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

    pub token_program: Program<'info, Token>,

    /// CHECK:
    #[account(mut)]
    pub token_account: UncheckedAccount<'info>,

    pub associated_token_program: Program<'info, associated_token::AssociatedToken>,

    /// CHECK:
    pub token_metadata_program: UncheckedAccount<'info>,

    #[account(
        init,
        payer = mint_authority,
        space = NftAccount::get_space(
            nft_name,
            ipfs_image_hash
        ),
        seeds = [
            b"nft".as_ref(),
            collection_pda.to_account_info().key.as_ref(),
            mint.to_account_info().key.as_ref()
        ],
        bump
    )]
    nft_pda: Box<Account<'info, NftAccount>>,

    /// CHECK:
    #[account(mut)]
    pub master_edition: UncheckedAccount<'info>,

    /// CHECK:
    #[account(mut)]
    pub metadata: UncheckedAccount<'info>,

    /// CHECK:
    #[account(mut)]
    pub collection_token_mint: UncheckedAccount<'info>,
    
    #[account(
        mut,
        seeds = [
            b"collection".as_ref(),
            collection_token_mint.to_account_info().key.as_ref()
        ],
        bump = collection_pda.bump
    )]
    collection_pda: Box<Account<'info, CollectionAccount>>,
    
    /// CHECK:
    #[account(mut)]
    pub collection_metadata: UncheckedAccount<'info>,

    /// CHECK:
    #[account(mut)]
    pub collection_master_ed: UncheckedAccount<'info>,

    /// CHECK: account checked in CPI
    #[account(mut)]
    collection_authority_record: UncheckedAccount<'info>,
}

#[account]
#[derive(Default)]
pub struct CollectionAccount {
    pub owner: Pubkey,
    pub token_mint: Pubkey,
    pub name: String,
    pub symbol: String,
    pub ipfs_image_hash: String,
    pub bump: u8,
    pub created: i64,
}

impl CollectionAccount {
    fn get_space(
        name: String,
        symbol: String,
        ipfs_image_hash: String
    ) -> usize {
        return DISCRIMINATOR_LENGTH
            + PUBLIC_KEY_LENGTH
            + PUBLIC_KEY_LENGTH
            + Self::get_string_size(name)
            + Self::get_string_size(symbol)
            + Self::get_string_size(ipfs_image_hash)
            + U8_LENGTH
            + I64_LENGTH;
    }
    fn get_string_size(property: String) -> usize {
        return property.as_bytes().len() + STRING_PREFIX_LENGTH;
    }
}

#[account]
#[derive(Default)]
pub struct NftAccount {
    pub token_mint: Pubkey,
    pub collection_mint: Pubkey,
    pub name: String,
    pub ipfs_image_hash: String,
    pub created: i64,
}

impl NftAccount {
    fn get_space(
        name: String,
        ipfs_image_hash: String
    ) -> usize {
        return DISCRIMINATOR_LENGTH
            + PUBLIC_KEY_LENGTH
            + PUBLIC_KEY_LENGTH
            + PUBLIC_KEY_LENGTH
            + Self::get_string_size(name)
            + Self::get_string_size(ipfs_image_hash)
            + I64_LENGTH;
    }
    fn get_string_size(property: String) -> usize {
        return property.as_bytes().len() + STRING_PREFIX_LENGTH;
    }
}
