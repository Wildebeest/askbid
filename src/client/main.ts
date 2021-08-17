import os from "os";
import fs from "mz/fs";
import path from "path";
import yaml from "yaml";
import {
  Connection,
  Keypair,
  LAMPORTS_PER_SOL,
  TransactionInstruction,
  PublicKey,
  sendAndConfirmTransaction,
  Transaction,
  SystemProgram,
} from "@solana/web3.js";

async function createKeypairFromFile(filePath: string): Promise<Keypair> {
  const secretKeyString = await fs.readFile(filePath, { encoding: "utf8" });
  const secretKey = Uint8Array.from(JSON.parse(secretKeyString));
  return Keypair.fromSecretKey(secretKey);
}

const PROGRAM_PATH = path.resolve(__dirname, "../../dist/program");
const PROGRAM_KEYPAIR_PATH = path.join(PROGRAM_PATH, "askbid-keypair.json");

async function main() {
  const CONFIG_FILE_PATH = path.resolve(
    os.homedir(),
    ".config",
    "solana",
    "cli",
    "config.yml"
  );
  const configYml = await fs.readFile(CONFIG_FILE_PATH, { encoding: "utf8" });
  const config = yaml.parse(configYml);
  const rpcUrl = config.json_rpc_url;
  const connection = new Connection(rpcUrl, "confirmed");

  const payer = await createKeypairFromFile(config.keypair_path);

  const lamports = await connection.getBalance(payer.publicKey);

  console.log(`Hi! Current balance: ${lamports / LAMPORTS_PER_SOL} SOL`);

  const programKeypair = await createKeypairFromFile(PROGRAM_KEYPAIR_PATH);
  const programId = programKeypair.publicKey;
  const programInfo = await connection.getAccountInfo(programId);
  if (programInfo === null) {
    throw new Error("Program needs to be built and deployed");
  }
  if (!programInfo.executable) {
    throw new Error(`Program is not executable`);
  }
  console.log(`Using program ${programId.toBase58()}`);

  const GREETING_SEED = "hello";
  const greetedPubkey = await PublicKey.createWithSeed(
    payer.publicKey,
    GREETING_SEED,
    programId
  );

  const GREETING_SIZE = 0;
  const greetedAccount = await connection.getAccountInfo(greetedPubkey);
  if (greetedAccount === null) {
    const lamports = await connection.getMinimumBalanceForRentExemption(
      GREETING_SIZE
    );

    const transaction = new Transaction().add(
      SystemProgram.createAccountWithSeed({
        fromPubkey: payer.publicKey,
        basePubkey: payer.publicKey,
        seed: GREETING_SEED,
        newAccountPubkey: greetedPubkey,
        lamports,
        space: GREETING_SIZE,
        programId,
      })
    );
    await sendAndConfirmTransaction(connection, transaction, [payer]);
  }

  console.log("Saying hello to", greetedPubkey.toBase58());
  const instruction = new TransactionInstruction({
    keys: [{ pubkey: greetedPubkey, isSigner: false, isWritable: true }],
    programId,
    data: Buffer.alloc(0),
  });
  await sendAndConfirmTransaction(
    connection,
    new Transaction().add(instruction),
    [payer]
  );
}
main();
