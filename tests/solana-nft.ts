import * as anchor from '@project-serum/anchor';
import { Program } from '@project-serum/anchor';
import { SolanaNft } from '../target/types/solana_nft';
import { expect } from 'chai';

describe('Solana NFT', () => {

  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.SolanaChat as Program<SolanaNft>;

  it('', async () => {
    
  });
});
