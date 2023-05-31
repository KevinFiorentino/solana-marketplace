import * as anchor from '@project-serum/anchor';
import { Program, BN } from '@project-serum/anchor';
import { PublicKey, Keypair, SystemProgram, SYSVAR_RENT_PUBKEY, Transaction, ComputeBudgetProgram } from '@solana/web3.js';
import { getAssociatedTokenAddressSync, TOKEN_PROGRAM_ID } from '@solana/spl-token';
import { Metaplex } from '@metaplex-foundation/js'
import { SolanaNft } from '../target/types/solana_nft';
import { expect } from 'chai';

const TOKEN_METADATA_PROGRAM_ID = new PublicKey(
  'metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s'
);

// https://spl.solana.com/associated-token-account
const SPL_ASSOCIATED_TOKEN_ACCOUNT_PROGRAM_ID = new PublicKey(
  'ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL'
);

describe('Solana NFTs', () => {

  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.SolanaNft as Program<SolanaNft>;


  /* ******************************
         COLLECTION ACCOUNTS
  ****************************** */

  const collectionKP = Keypair.generate();
  console.log('collectionKP', collectionKP.publicKey.toString());

  const collectionATA = getAssociatedTokenAddressSync(
    collectionKP.publicKey,
    provider.wallet.publicKey,
  );

  const [collectionPDA] = anchor.web3.PublicKey
    .findProgramAddressSync(
      [
        Buffer.from('collection'),
        provider.wallet.publicKey.toBuffer(),
        collectionKP.publicKey.toBuffer(),
      ],
      program.programId
    );
  console.log('collectionPDA', collectionPDA.toString());

  const [collectionMetadataPDA] = anchor.web3.PublicKey
    .findProgramAddressSync(
      [
        Buffer.from('metadata'),
        TOKEN_METADATA_PROGRAM_ID.toBuffer(),
        collectionKP.publicKey.toBuffer(),
      ],
      TOKEN_METADATA_PROGRAM_ID
    );

  const [collectionMasterEditionPDA] = anchor.web3.PublicKey
    .findProgramAddressSync(
      [
        Buffer.from('metadata'),
        TOKEN_METADATA_PROGRAM_ID.toBuffer(),
        collectionKP.publicKey.toBuffer(),
        Buffer.from('edition'),
      ],
      TOKEN_METADATA_PROGRAM_ID
    );

  const [collectionAuthorityRecordPDA] = anchor.web3.PublicKey
    .findProgramAddressSync(
      [
        Buffer.from('metadata'),
        TOKEN_METADATA_PROGRAM_ID.toBuffer(),
        collectionKP.publicKey.toBuffer(),
        Buffer.from('collection_authority'),
        collectionPDA.toBuffer(),
      ],
      TOKEN_METADATA_PROGRAM_ID
    );


  /* ******************************
            NFT ACCOUNTS
  ****************************** */

  const nftKP = Keypair.generate();
  console.log('nftKP', nftKP.publicKey.toString());

  const nftATA = getAssociatedTokenAddressSync(
    nftKP.publicKey,
    provider.wallet.publicKey,
  );

  const [nftPDA] = anchor.web3.PublicKey
    .findProgramAddressSync(
      [
        Buffer.from('nft'),
        collectionPDA.toBuffer(),
        nftKP.publicKey.toBuffer(),
      ],
      program.programId
    );
  console.log('nftPDA', nftPDA.toString());

  const [nftMetadataPDA] = anchor.web3.PublicKey
    .findProgramAddressSync(
      [
        Buffer.from('metadata'),
        TOKEN_METADATA_PROGRAM_ID.toBuffer(),
        nftKP.publicKey.toBuffer(),
      ],
      TOKEN_METADATA_PROGRAM_ID
    );

  const [nftMasterEditionPDA] = anchor.web3.PublicKey
    .findProgramAddressSync(
      [
        Buffer.from('metadata'),
        TOKEN_METADATA_PROGRAM_ID.toBuffer(),
        nftKP.publicKey.toBuffer(),
        Buffer.from('edition'),
      ],
      TOKEN_METADATA_PROGRAM_ID
    );



  /* ******************************
            COLLECTIONS
  ****************************** */

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
        'https://arweave.net/l0Vjj3rZKQm-FVbCCj2OH15YMWAveUseuCLGkcPE-x0',    // Image URI
        'https://arweave.net/mF0bbubycS50wu2-WSkZoU2g5scupj0hfzk8eqFEtpA',    // Metadata URI
      )
      .accounts({
        mint: collectionKP.publicKey,
        mintAuthority: provider.wallet.publicKey,
        payer: provider.wallet.publicKey,
        rent: SYSVAR_RENT_PUBKEY,
        systemProgram: SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
        tokenAccount: collectionATA,
        associatedTokenProgram: SPL_ASSOCIATED_TOKEN_ACCOUNT_PROGRAM_ID,
        tokenMetadataProgram: TOKEN_METADATA_PROGRAM_ID,
        masterEdition: collectionMasterEditionPDA,
        metadata: collectionMetadataPDA,
        collectionAuthorityRecord: collectionAuthorityRecordPDA,
        collectionPda: collectionPDA,
      })
      .instruction();

    t.add(i);

    const latestBlockHash = await provider.connection.getLatestBlockhash();
    t.recentBlockhash = latestBlockHash.blockhash;
    t.lastValidBlockHeight = latestBlockHash.lastValidBlockHeight;

    t.feePayer = provider.wallet.publicKey;
    t.sign(collectionKP);

    const tSigned = await provider.wallet.signTransaction(t);
    const tx = await provider.connection.sendRawTransaction(tSigned.serialize());
    const con = await provider.connection.confirmTransaction(tx);

    console.log('tx confirm', con);
  });

  it('Get all collections', async () => {
    const collections = await program.account.collectionAccount.all();
    // console.log('collections', collections);
    expect(1).equal(collections.length);
  });

  it('Paginate collections by owner', async () => {

    const collectionClient = program.account.collectionAccount;
    const accountName = (collectionClient as any)._idlAccount.name;

    const accountDiscriminatorFilter = {
      memcmp: collectionClient.coder.accounts.memcmp(accountName)
    };
    const ownerFilter = {
      memcmp: {
        bytes: provider.wallet.publicKey.toBase58(),
        offset: 8
      }
    };

    const rawCollections = await provider.connection.getProgramAccounts(program.programId, {
      filters: [accountDiscriminatorFilter, ownerFilter],
      dataSlice: { offset: 0, length: 0 },
    });

    const collectionsPDAs = [];
    rawCollections.forEach(c => {
      collectionsPDAs.push(c.pubkey);
    });

    const collections = await program.account.collectionAccount.fetchMultiple(collectionsPDAs);
    expect(1).equal(collections.length);
  });

  

  /* ******************************
                NFTs
  ****************************** */

  it('Mint NFT', async () => {

    const t = new Transaction();

    const modifyComputeUnits = ComputeBudgetProgram.setComputeUnitLimit({ 
      units: 300000 
    });
    t.add(modifyComputeUnits);

    const i = await program.methods
      .mintNftFromCollection(
        'First NFT',
        'https://arweave.net/l0Vjj3rZKQm-FVbCCj2OH15YMWAveUseuCLGkcPE-x0',    // Image URI
        'https://arweave.net/mF0bbubycS50wu2-WSkZoU2g5scupj0hfzk8eqFEtpA',    // Metadata URI
      )
      .accounts({
        mint: nftKP.publicKey,
        mintAuthority: provider.wallet.publicKey,
        payer: provider.wallet.publicKey,
        rent: SYSVAR_RENT_PUBKEY,
        systemProgram: SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
        tokenAccount: nftATA,
        associatedTokenProgram: SPL_ASSOCIATED_TOKEN_ACCOUNT_PROGRAM_ID,
        tokenMetadataProgram: TOKEN_METADATA_PROGRAM_ID,
        nftPda: nftPDA,
        masterEdition: nftMasterEditionPDA,
        metadata: nftMetadataPDA,
        collectionTokenMint: collectionKP.publicKey,
        collectionPda: collectionPDA,
        collectionMetadata: collectionMetadataPDA,
        collectionMasterEd: collectionMasterEditionPDA,
        collectionAuthorityRecord: collectionAuthorityRecordPDA,
      })
      .instruction();

    t.add(i);

    const latestBlockHash = await provider.connection.getLatestBlockhash();
    t.recentBlockhash = latestBlockHash.blockhash;
    t.lastValidBlockHeight = latestBlockHash.lastValidBlockHeight;

    t.feePayer = provider.wallet.publicKey;
    t.sign(nftKP);

    const tSigned = await provider.wallet.signTransaction(t);
    const tx = await provider.connection.sendRawTransaction(tSigned.serialize());
    const con = await provider.connection.confirmTransaction(tx);

    console.log('tx confirm', con);
  });

  it('Paginate NFTs by collection mint', async () => {
    
    const nftClient = program.account.nftAccount;
    const accountName = (nftClient as any)._idlAccount.name;

    const accountDiscriminatorFilter = {
      memcmp: nftClient.coder.accounts.memcmp(accountName)
    };
    const collectionFilter = {
      memcmp: {
        bytes: collectionKP.publicKey.toBase58(),
        offset: 40
      }
    };

    const rawNfts = await provider.connection.getProgramAccounts(program.programId, {
      filters: [accountDiscriminatorFilter, collectionFilter],
      dataSlice: { offset: 0, length: 0 },
    });

    const nftsPDAs = [];
    rawNfts.forEach(n => {
      nftsPDAs.push(n.pubkey);
    });

    const nfts = await program.account.nftAccount.fetchMultiple(nftsPDAs);
    expect(1).equal(nfts.length);
  });

  it('Get NFT by mint', async () => {
    const metaplex = Metaplex.make(provider.connection);
    const nft = await metaplex.nfts().findByMint({ mintAddress: nftKP.publicKey });
    console.log(nft);
    expect(nftKP.publicKey.toString()).equal(nft.address.toString());
  });

});
