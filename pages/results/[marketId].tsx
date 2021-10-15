import {useRouter} from 'next/router';
import Head from 'next/head';
import {useEffect, useState} from "react";
import {
    Connection,
    AccountInfo,
    KeyedAccountInfo,
    Context,
    PublicKey, TransactionInstruction, Transaction,
} from "@solana/web3.js";
import * as borsh from 'borsh';
import {
    SearchMarketAccount,
    SearchMarketAccountSchema,
    PROGRAM_ID,
    ResultAccountSchema,
    ResultAccount, OrderSchema, Order, LAMPORTS_PER_TOKEN, Instruction, InstructionSchema, Decide
} from "../../lib/client";
import {getProvider} from "../../lib/phantom";

function Result(props: { result: ResultAccount, pubKey: PublicKey, bestResult: PublicKey, lowestAsk: Order | undefined, connection: Connection, onBestResultChange: (PublicKey) => void }) {
    const router = useRouter();
    const onDecide = async () => {
        if (props.bestResult.toString() !== PublicKey.default.toString()) {
            return;
        }

        const connection = props.connection;
        const provider = getProvider();
        const marketPublicKey = new PublicKey(router.query.marketId);
        const decisionData = borsh.serialize(InstructionSchema, new Instruction({
            instruction: "Decide",
            Decide: new Decide(),
        }));
        const decideInstruction = new TransactionInstruction({
            keys: [
                {
                    pubkey: marketPublicKey,
                    isSigner: false,
                    isWritable: true,
                },
                {
                    pubkey: provider.publicKey,
                    isSigner: true,
                    isWritable: false
                },
                {
                    pubkey: props.pubKey,
                    isSigner: false,
                    isWritable: false
                }
            ], programId: PROGRAM_ID, data: Buffer.from(decisionData)
        });
        const recentBlockhash = (await connection.getRecentBlockhash()).blockhash;
        const transaction = (new Transaction({recentBlockhash, feePayer: provider.publicKey}))
            .add(decideInstruction);
        const signedTransaction = await provider.signTransaction(transaction);
        const signature = await connection.sendRawTransaction(signedTransaction.serialize());
        await connection.confirmTransaction(signature, 'confirmed');
        props.onBestResultChange(props.pubKey);
    };

    let probability = "--%";
    if (props.lowestAsk) {
        probability = `${(props.lowestAsk.price.toNumber() / LAMPORTS_PER_TOKEN * 100).toFixed(2)}%`;
    }

    let decideButton = null;
    if (props.bestResult.toString() === PublicKey.default.toString())
    {
        decideButton = (<button
            className="border rounded bg-green-50 border-green-100 hover:bg-green-200 hover:border-green-300"
            onClick={onDecide}>⭐️
        </button>);
    } else if (props.bestResult.toString() === props.pubKey.toString()) {
        decideButton = (<button
            className="border rounded bg-green-200 border-green-300">⭐️
        </button>);
    } else {
        decideButton = (<button
            className="border rounded bg-green-50 border-green-100 opacity-50" disabled>⭐️
        </button>);
    }

    return (
        <div className="py-2 flex">
            <div className="mr-4 text-center flex flex-col w-12">
                {decideButton}
                <div className="text-gray-500">{probability}</div>
            </div>
            <div>
                <a href={props.result.url} className="text-l font-semibold text-blue-600" target="_blank" rel="noreferrer">
                    {props.result.name}
                </a>
                <div>{props.result.snippet}</div>
            </div>
        </div>
    );
}

export default function Results() {
    const router = useRouter();
    const [connection, setConnection] = useState<Connection>(new Connection("http://127.0.0.1:8899", 'confirmed'));
    const [searchMarket, setSearchMarket] = useState<SearchMarketAccount>();
    const [query, setQuery] = useState<string>("");
    const [resultAccounts, setResultAccounts] = useState<Map<string, ResultAccount>>(new Map());
    const [bestResult, setBestResult] = useState<PublicKey>(PublicKey.default);
    const [lowestAsks, setLowestAsks] = useState<Map<string, Order>>(new Map());

    const onResultChange = (keyedAccountInfo: KeyedAccountInfo) => {
        const account = borsh.deserialize(ResultAccountSchema, ResultAccount, keyedAccountInfo.accountInfo.data);
        setResultAccounts(resultAccounts =>
            resultAccounts.set(keyedAccountInfo.accountId.toString(), account)
        );
    };

    const onOrderChange = (keyedAccountInfo: KeyedAccountInfo) => {
        const order = borsh.deserialize(OrderSchema, Order, keyedAccountInfo.accountInfo.data);
        const resultPubkey = new PublicKey(order.result);
        const currentAsk = lowestAsks.get(resultPubkey.toString());
        if (currentAsk?.price < order.price) {
            return;
        }
        setLowestAsks((lowestAsks) => {
            let newAsks = new Map(lowestAsks);
            newAsks.set(resultPubkey.toString(), order);
            return newAsks;
        });
    };

    const onProgramAccountChange = (keyedAccountInfo: KeyedAccountInfo) => {
        if (keyedAccountInfo.accountInfo.data[0] === 1) {
            onResultChange(keyedAccountInfo);
        } else if (keyedAccountInfo.accountInfo.data[0] === 2) {
            onOrderChange(keyedAccountInfo);
        }
    };

    const onSortClick = () => {
        setResultAccounts(resultAccounts => {
            let entries = Array.from(resultAccounts.entries());
            entries.sort((a, b) =>
                lowestAsks.get(b[0]).price.toNumber() - lowestAsks.get(a[0]).price.toNumber());
            return new Map(entries);
        });
    };

    useEffect(() => {
        (async () => {
            const {marketId} = router.query;
            if (!marketId) {
                return;
            }

            const marketPublicKey = new PublicKey(marketId);
            const accountInfo = await connection.getAccountInfo(marketPublicKey);
            const account = borsh.deserialize(SearchMarketAccountSchema, SearchMarketAccount, accountInfo.data);
            setSearchMarket(account);
            setBestResult(new PublicKey(account.best_result));
            setQuery(account.search_string);

            const filters = [
                {
                    memcmp: {
                        offset: 2,
                        bytes: marketPublicKey.toString()
                    }
                }];
            connection.onProgramAccountChange(PROGRAM_ID, onProgramAccountChange, 'confirmed', filters);
            const accounts = await connection.getProgramAccounts(PROGRAM_ID, {
                commitment: 'confirmed', filters
            });
            for (const account of accounts) {
                onProgramAccountChange({accountId: account.pubkey, accountInfo: account.account});
            }
        })();
    }, [router, connection]);
    return (
        <div>
            <Head>
                <title>🚀 - {searchMarket?.search_string}</title>
                <link rel="icon" href="/favicon.ico"/>
            </Head>
            <div className="flex">
                <div className="pt-8 pr-5 pl-2 text-xl">🚀 AskBid 🌚</div>
                <div className="flex border border-gray-200 rounded m-4 p-4 shadow text-xl flex-1">
                    <div>🔎</div>
                    <input type="text" className="w-full outline-none px-3" name="query" value={query}
                           autoComplete="off"
                           onChange={(event) => setQuery(event.target.value)}/>
                    <div>🇺🇸</div>
                </div>
            </div>
            <div className="pl-2 pr-4">
                <div className="text-right text-sm"><a className="text-blue-600" onClick={onSortClick}>Sort</a></div>
                {Array.from(resultAccounts.entries()).map((entry) => {
                    const [pubkey, result] = entry;
                    return <Result result={result} key={pubkey} pubKey={new PublicKey(pubkey)} bestResult={bestResult}
                                   lowestAsk={lowestAsks.get(pubkey)} connection={connection} onBestResultChange={setBestResult}/>;
                })}
            </div>
        </div>
    );
}