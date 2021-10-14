import {useRouter} from 'next/router';
import Head from 'next/head';
import {useEffect, useState} from "react";
import {
    Connection,
    AccountInfo,
    KeyedAccountInfo,
    Context,
    PublicKey,
} from "@solana/web3.js";
import * as borsh from 'borsh';
import {
    SearchMarketAccount,
    SearchMarketAccountSchema,
    PROGRAM_ID,
    ResultAccountSchema,
    ResultAccount
} from "../../lib/client";

function Result(props: { result: ResultAccount }) {
    return (
        <div className="py-2 flex">
            <div className="mr-4 text-center flex flex-col w-8">
                <button
                    className="border rounded bg-green-50 border-green-100 hover:bg-green-200 hover:border-green-300">😍
                </button>
            </div>
            <div>
                <a href={props.result.url} className="text-l font-semibold text-blue-600">
                    {props.result.name}
                </a>
                <div>{props.result.snippet}</div>
            </div>
        </div>
    );
}

export default function Results() {
    const router = useRouter();
    const [searchMarket, setSearchMarket] = useState<SearchMarketAccount>();
    const [query, setQuery] = useState<string>("");
    const [resultAccounts, setResultAccounts] = useState<{ pubkey: PublicKey, account: ResultAccount }[]>([]);

    const onProgramAccountChange = (keyedAccountInfo: KeyedAccountInfo, _context: Context) => {
        console.log(keyedAccountInfo);
        const account = borsh.deserialize(ResultAccountSchema, ResultAccount, keyedAccountInfo.accountInfo.data);
        setResultAccounts(resultAccounts => [...resultAccounts, {pubkey: keyedAccountInfo.accountId, account}]);
    };

    useEffect(() => {
        const connection = new Connection("http://127.0.0.1:8899", 'confirmed');
        (async () => {
            const {marketId} = router.query;
            if (!marketId) {
                return;
            }

            const marketPublicKey = new PublicKey(marketId);
            const accountInfo = await connection.getAccountInfo(marketPublicKey);
            const account = borsh.deserialize(SearchMarketAccountSchema, SearchMarketAccount, accountInfo.data);
            setSearchMarket(account);
            setQuery(account.search_string);
            const resultFilters = [
                {
                    memcmp: {
                        offset: 2,
                        bytes: marketPublicKey.toString()
                    }
                }
            ];
            connection.onProgramAccountChange(PROGRAM_ID, onProgramAccountChange, 'confirmed', resultFilters);
            const results = await connection.getProgramAccounts(PROGRAM_ID, {commitment: 'confirmed', filters: resultFilters});
            const newResultAccounts = results.map(
                (result) => {
                    return {
                        pubkey: result.pubkey,
                        account: borsh.deserialize(ResultAccountSchema, ResultAccount, result.account.data)
                    };
                }
            );
            setResultAccounts(newResultAccounts);
        })();
    }, [router]);
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
                           onChange={(event) => setQuery(event.target.value)}/>
                    <div>🇺🇸</div>
                </div>
            </div>
            <div className="pl-2 pr-4">
                {resultAccounts.map((result) =>
                    <Result result={result.account} key={result.pubkey.toString()}/>)}
            </div>
        </div>
    );
}