import * as anchor from '@project-serum/anchor';
import { Program, BN } from '@project-serum/anchor';
import { PublicKey, Keypair, SystemProgram, SYSVAR_RENT_PUBKEY, Transaction, ComputeBudgetProgram } from '@solana/web3.js';
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

  const mintKeypair = Keypair.generate();
  console.log('mintKeypair', mintKeypair.publicKey.toString());

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
        mintKeypair.publicKey.toBuffer(),
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



  it('Mint collection', async () => {

    const t = new Transaction();

    const modifyComputeUnits = ComputeBudgetProgram.setComputeUnitLimit({ 
      units: 300000 
    });
    t.add(modifyComputeUnits);

    const i = await program.methods
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
        tokenProgram: TOKEN_PROGRAM_ID,
        tokenAccount: ATA,
        associatedTokenProgram: SPL_ASSOCIATED_TOKEN_ACCOUNT_PROGRAM_ID,
        tokenMetadataProgram: TOKEN_METADATA_PROGRAM_ID,
        masterEdition: masterEditionPDA,
        metadata: metadataPDA,
        collectionAuthorityRecord: collectionAuthorityRecordPDA,
        collectionPda: collectionPDA,
      })
      .instruction();

    t.add(i);

    const latestBlockHash = await provider.connection.getLatestBlockhash();
    t.recentBlockhash = latestBlockHash.blockhash;
    t.lastValidBlockHeight = latestBlockHash.lastValidBlockHeight;

    t.feePayer = provider.wallet.publicKey;
    t.sign(mintKeypair);

    const tSigned = await provider.wallet.signTransaction(t);
    const tx = await provider.connection.sendRawTransaction(tSigned.serialize());
    const con = await provider.connection.confirmTransaction(tx);

    console.log('tx confirm', con);
  });

  it('Get colletions', async () => {
    const collections = await program.account.collectionPdaAccount.all();
    console.log('collections', collections);
    expect(1).equal(collections.length);
  });

  it('Get colletions by owner', async () => {
    const collections = await program.account.collectionPdaAccount.all([
      {
        memcmp: {
          bytes: provider.wallet.publicKey.toBase58(),
          offset: 8
        },
      },
    ]);
    console.log('collections', collections);
    expect(1).equal(collections.length);
  });

});
