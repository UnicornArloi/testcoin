import * as anchor from '@project-serum/anchor';
import { Program } from '@project-serum/anchor';
import { TestCoin } from '../target/types/test_coin';

describe('test-coin', () => {
  anchor.setProvider(anchor.AnchorProvider.env());
  const program = anchor.workspace.TestCoin as Program<TestCoin>;

  it('Is initialized!', async () => {
    const tx = await program.methods.initialize(new anchor.BN(500), new anchor.BN(500)).rpc();
    console.log("Transaction signature", tx);
  });
});
