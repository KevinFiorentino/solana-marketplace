import * as anchor from '@project-serum/anchor';
import { Program, BN } from '@project-serum/anchor';
import { PublicKey, Keypair, SystemProgram, SYSVAR_RENT_PUBKEY } from '@solana/web3.js';
import { getAssociatedTokenAddressSync, TOKEN_PROGRAM_ID } from '@solana/spl-token';
import { SolanaNft } from '../target/types/solana_nft';
import { expect } from 'chai';

const TOKEN_METADATA_PROGRAM_ID = new PublicKey(
  'metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s'
);

// https://spl.solana.com/associated-token-account
const SPL_ASSOCIATED_TOKEN_ACCOUNT_PROGRAM_ID = new PublicKey(
  'ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL'
);

describe('Solana NFT', () => {

  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.SolanaNft as Program<SolanaNft>;

  const [userPDA] = anchor.web3.PublicKey
    .findProgramAddressSync(
      [
        Buffer.from('user_account'),
        provider.wallet.publicKey.toBuffer(),
      ],
      program.programId
    );

  const mintKeypair = Keypair.generate();

  const ATA = getAssociatedTokenAddressSync(
    mintKeypair.publicKey,
    provider.wallet.publicKey,
  );

  const [metadataPDA] = anchor.web3.PublicKey
    .findProgramAddressSync(
      [
        Buffer.from('metadata'),
        TOKEN_METADATA_PROGRAM_ID.toBuffer(),
        mintKeypair.publicKey.toBuffer(),
      ],
      TOKEN_METADATA_PROGRAM_ID
    );

  const [masterEditionPDA] = anchor.web3.PublicKey
    .findProgramAddressSync(
      [
        Buffer.from('metadata'),
        TOKEN_METADATA_PROGRAM_ID.toBuffer(),
        mintKeypair.publicKey.toBuffer(),
        Buffer.from('edition'),
      ],
      TOKEN_METADATA_PROGRAM_ID
    );

  const [collectionPDA] = anchor.web3.PublicKey
    .findProgramAddressSync(
      [
        Buffer.from('collection'),
        provider.wallet.publicKey.toBuffer(),
        new BN(0).toArrayLike(Buffer, 'le', 8),
      ],
      program.programId
    );

  const [collectionAuthorityRecordPDA] = anchor.web3.PublicKey
    .findProgramAddressSync(
      [
        Buffer.from('metadata'),
        TOKEN_METADATA_PROGRAM_ID.toBuffer(),
        mintKeypair.publicKey.toBuffer(),
        Buffer.from('collection_authority'),
        collectionPDA.toBuffer(),
      ],
      TOKEN_METADATA_PROGRAM_ID
    );




  it('Create user', async () => {
    const tx = await program.methods
      .createUserAccount()
      .accounts({
        userPda: userPDA,
        rent: SYSVAR_RENT_PUBKEY,
        payer: provider.wallet.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .rpc();
    console.log('tx', tx);
  });

  it('Get users', async () => {
    const users = await program.account.userPda.all();
    console.log('users', users);
    expect(1).equal(users.length);
  });

  it('Mint collection', async () => {
    const tx = await program.methods
      .mintCollection(
        'My First Collection',
        'MFC',
        'https://arweave.net/mF0bbubycS50wu2-WSkZoU2g5scupj0hfzk8eqFEtpA',
        'https://arweave.net/l0Vjj3rZKQm-FVbCCj2OH15YMWAveUseuCLGkcPE-x0',
      )
      .accounts({
        mint: mintKeypair.publicKey,
        mintAuthority: provider.wallet.publicKey,
        payer: provider.wallet.publicKey,
        rent: SYSVAR_RENT_PUBKEY,
        systemProgram: SystemProgram.programId,
        userPda: userPDA,
        tokenProgram: TOKEN_PROGRAM_ID,
        tokenAccount: ATA,
        associatedTokenProgram: SPL_ASSOCIATED_TOKEN_ACCOUNT_PROGRAM_ID,
        tokenMetadataProgram: TOKEN_METADATA_PROGRAM_ID,
        masterEdition: masterEditionPDA,
        metadata: metadataPDA,
        collectionAuthorityRecord: collectionAuthorityRecordPDA,
        collectionPda: collectionPDA,
      })
      .signers([mintKeypair])
      .rpc();
    console.log('tx2', tx);
  });

  it('Get colletions', async () => {
    const collections = await program.account.collectionPdaAccount.all();
    console.log('collections', collections);
    // expect(1).equal(users.length);
  });

});
