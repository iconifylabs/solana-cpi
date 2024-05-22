import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { PublicKey, SystemProgram } from "@solana/web3.js";
import { assert } from "chai";
import { Test } from "../target/types/test";
import { Receiver } from "../target/types/receiver";

describe("inter-contract call", () => {
  const provider = anchor.AnchorProvider.local();
  anchor.setProvider(provider);

  const callerProgram = anchor.workspace.Test as Program<Test>;
  const receiverProgram = anchor.workspace.Receiver as Program<Receiver>;

  let callerStatePDA: PublicKey;
  let receiverStatePDA: PublicKey;
  let callerStateBump: number;
  let receiverStateBump: number;

  let user = anchor.web3.Keypair.generate();

  const airdrop = async (publicKey: anchor.web3.PublicKey) => {
    const airdropSignature = await provider.connection.requestAirdrop(
      publicKey,
      anchor.web3.LAMPORTS_PER_SOL,// Adjust amount as necessary
    );
    await provider.connection.confirmTransaction(airdropSignature);
  }

  const message = "Hello from Caller!";

  before(async () => {
    await airdrop(user.publicKey);

    // Find PDA for caller state
    [callerStatePDA, callerStateBump] = await PublicKey.findProgramAddress(
      [Buffer.from("state")],
      callerProgram.programId
    );

    // Find PDA for receiver state
    [receiverStatePDA, receiverStateBump] = await PublicKey.findProgramAddress(
      [Buffer.from("state")],
      receiverProgram.programId
    );
  });

  const getTxnLogs = async (tx) => {
    const confirmation = await provider.connection.confirmTransaction(tx, "confirmed");
    console.log("Transaction confirmation status:", confirmation.value.err);

    let txDetails = await provider.connection.getTransaction(tx, { commitment: "confirmed" })

    if (txDetails?.meta?.logMessages) {
      txDetails.meta.logMessages.forEach(log => {
        console.log("Log:", log);
      });
    }

  }

  it("Initializes the caller state", async () => {
    await callerProgram.rpc.initialize(new anchor.BN(100), {
      accounts: {
        state: callerStatePDA,
        user: user.publicKey,
        systemProgram: SystemProgram.programId,
      },
      signers: [user],
    });

    let state = await callerProgram.account.callerState.fetch(callerStatePDA);
    assert.equal(state.fees.toString(), "100");
    // });

    // it("Initializes the receiver state", async () => {
    await receiverProgram.rpc.initialize(callerStatePDA, {
      accounts: {
        state: receiverStatePDA,
        user: user.publicKey,
        systemProgram: SystemProgram.programId,
      },
      signers: [user],
    });

    let statex = await receiverProgram.account.receiverState.fetch(receiverStatePDA);
    assert.equal(statex.xcall.toBase58(), callerStatePDA.toBase58());
    // });

    // it("Calls the receiver method from the caller", async () => {
    let tx = await callerProgram.rpc.callReceiverMethod(message, {
      accounts: {
        state: callerStatePDA,
        user: user.publicKey,
        systemProgram: SystemProgram.programId,
      },
      remainingAccounts: [
        {
          pubkey: receiverStatePDA,
          isWritable: true,
          isSigner: false,
        },
        {
          pubkey: receiverProgram.programId,
          isWritable: false,
          isSigner: false,
        },
      ],
      signers: [user],
    }).catch(e => console.log(e))

    getTxnLogs(tx)

  });
});
