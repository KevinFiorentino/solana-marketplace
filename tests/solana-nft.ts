import * as anchor from '@project-serum/anchor';
import { Program } from '@project-serum/anchor';
import { SYSVAR_RENT_PUBKEY } from "@solana/web3.js";
import { SolanaNft } from '../target/types/solana_nft';
import { expect } from 'chai';

const { SystemProgram } = anchor.web3;

describe('Solana NFT', () => {

  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.SolanaNft as Program<SolanaNft>;

  const [userPDA] = anchor.web3.PublicKey
    .findProgramAddressSync(
      [
        Buffer.from('user_account'),
        provider.wallet.publicKey.toBytes()
      ],
      program.programId
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
  });

});
