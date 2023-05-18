use {
    anchor_lang::{
        prelude::*, solana_program::program::invoke, solana_program::program::invoke_signed,
        system_program, AnchorDeserialize, AnchorSerialize,
    },
    anchor_spl::{associated_token, token, token::Token},
    mpl_token_metadata::instruction::{
        approve_collection_authority, create_master_edition_v3, create_metadata_accounts_v3,
        set_and_verify_collection, sign_metadata, update_metadata_accounts_v2,
    },
    mpl_token_metadata::state::Creator,
};

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[program]
pub mod solana_nft {
    use super::*;

}


