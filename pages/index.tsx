import Head from 'next/head';
import {useState, useEffect} from "react";
import {
    Connection,
    Keypair,
    TransactionInstruction,
    Transaction,
    SystemProgram,
    PublicKey,
} from "@solana/web3.js";
import * as borsh from 'borsh';

function SearchButton(props) {
    let hoverStates = "opacity-50";
    if (props.isConnected) {
        hoverStates = "hover:bg-green-400 hover:border-green-500";
    }
    return (
        <button
            className={"mr-3 bg-green-200 border border-green-300 py-3 px-4 rounded " + hoverStates}
            onClick={props.onClick}>
            Ask the internet
        </button>
    );
}

interface ConnectOpts {
    onlyIfTrusted: boolean;
}

type PhantomEvent = "disconnect" | "connect";

interface PhantomProvider {
    publicKey?: PublicKey;
    isConnected?: boolean;
    signTransaction: (transaction: Transaction) => Promise<Transaction>;
    on: (event: PhantomEvent, handler: (args: any) => void) => void;
    connect: (opts?: Partial<ConnectOpts>) => Promise<{ publicKey: PublicKey }>;
}

const getProvider = (): PhantomProvider | undefined => {
    if (!process.browser) {
        return;
    }
    if ("solana" in window) {
        const anyWindow: any = window;
        const provider = anyWindow.solana;
        if (provider.isPhantom) {
            return provider;
        }
    }
    window.open("https://phantom.app/", "_blank");
};


function WalletButton(props) {
    const provider = getProvider();
    useEffect(() => {
        if (!provider) {
            return;
        }

        provider.on("connect", () => {
            props.setConnected(true);
        });
        provider.connect({onlyIfTrusted: true});
    }, [provider]);

    const onClick = () => {
        provider.connect();
    }

    return (
        <button
            onClick={onClick}
            className="mr-3 bg-purple-200 border border-purple-300 py-3 px-4 rounded hover:bg-purple-400 hover:border-purple-500">
            Setup a wallet
        </button>
    );
}

const PROGRAM_ID = new PublicKey("CtRJbPMscDFRJptvh6snF5GJXDNCJHMFsfYoczds37AV");

class Instruction {
    CreateMarket: CreateMarket;
    instruction: string;

    constructor(fields) {
        this.CreateMarket = fields.CreateMarket;
        this.instruction = fields.instruction;
    }
}

class CreateMarket {
    expires_slot: number;
    search_string: string;

    constructor(expires_slot, search_string) {
        this.expires_slot = expires_slot;
        this.search_string = search_string;
    }
}

const InstructionWrapperSchema = [Instruction, {
    kind: 'enum',
    field: 'instruction',
    values: [['CreateMarket', {}]]
}];
const CreateMarketSchema = [CreateMarket, {
    kind: 'struct',
    fields: [['expires_slot', 'u64'], ['search_string', 'string']]
}];

class SearchMarketAccount {
    decision_authority: Uint8Array;
    search_string: string;
    best_result: Uint8Array;
    expires_slot: number;

    constructor(fields: { decision_authority: PublicKey, search_string: string, best_result: PublicKey, expires_slot: number }) {
        this.decision_authority = fields.decision_authority.toBytes();
        this.search_string = fields.search_string;
        this.best_result = fields.best_result.toBytes();
        this.expires_slot = fields.expires_slot;
    }
}

const SearchMarketAccountSchema: borsh.Schema = new Map([[SearchMarketAccount, {
    kind: 'struct',
    fields: [
        ['decision_authority', [32]],
        ['search_string', 'string'],
        ['best_result', [32]],
        ['expires_slot', 'u64']],
}]]);

// @ts-ignore
const InstructionSchema: borsh.Schema = new Map([
    InstructionWrapperSchema,
    CreateMarketSchema]);

export default function Home() {
    const [isConnected, setConnected] = useState<boolean>(false);
    const [query, setQuery] = useState<string>("");
    const connection = new Connection("http://127.0.0.1:8899", 'confirmed');

    const onQueryChange = event => {
        setQuery(event.target.value);
    };

    const submitSearch = async event => {
        const provider = getProvider();
        const epochInfo = await connection.getEpochInfo();
        const slot = epochInfo.absoluteSlot + 25;
        const data = borsh.serialize(InstructionSchema, new Instruction({
            instruction: "CreateMarket",
            CreateMarket: new CreateMarket(slot, query),
        }));
        const searchMarketAccount = new SearchMarketAccount({
            decision_authority: provider.publicKey,
            search_string: query,
            expires_slot: slot,
            best_result: PublicKey.default
        });
        const accountSize = borsh.serialize(SearchMarketAccountSchema, searchMarketAccount).byteLength;
        const rentExemptAmount = await connection.getMinimumBalanceForRentExemption(accountSize);
        const marketAccountKey = Keypair.generate();
        const newAccountInstruction = SystemProgram.createAccount({
            fromPubkey: provider.publicKey,
            programId: PROGRAM_ID,
            newAccountPubkey: marketAccountKey.publicKey,
            lamports: rentExemptAmount,
            space: accountSize
        });

        const transactionInstruction = new TransactionInstruction({
            keys: [
                {
                    pubkey: marketAccountKey.publicKey,
                    isSigner: false,
                    isWritable: true,
                },
                {
                    pubkey: provider.publicKey,
                    isSigner: true,
                    isWritable: false
                }
            ], programId: PROGRAM_ID, data: Buffer.from(data)
        });
        const recentBlockhash = (await connection.getRecentBlockhash()).blockhash;
        const transaction = (new Transaction({recentBlockhash, feePayer: provider.publicKey}))
            .add(newAccountInstruction)
            .add(transactionInstruction);
        transaction.partialSign(marketAccountKey);
        const signedTransaction = await provider.signTransaction(transaction);
        const signature = await connection.sendRawTransaction(signedTransaction.serialize());
        await connection.confirmTransaction(signature);
        console.log(`Created new account ${marketAccountKey.publicKey}`);
    };
    return (
        <div>
            <Head>
                <title>🚀 AskBid 🔎</title>
                <link rel="icon" href="/favicon.ico"/>
            </Head>

            <main>
                <div className="flex justify-center pt-20">
                    <div>
                        <h1 className="mb-6 text-5xl">🚀 AskBid: A Search Exchange 🌚</h1>

                        <div className="flex border border-gray-200 rounded p-4 shadow text-xl">
                            <div>🔎</div>
                            <input type="text" className="w-full outline-none px-3" name="query" required
                                   onChange={onQueryChange} value={query}/>
                            <div>🇺🇸</div>
                        </div>

                        <div className="mt-8 text-center">
                            {!isConnected && <WalletButton setConnected={setConnected}/>}
                            <SearchButton isConnected={isConnected} setConnected={setConnected} onClick={submitSearch}/>
                        </div>
                    </div>
                </div>

            </main>
        </div>
    )
}
