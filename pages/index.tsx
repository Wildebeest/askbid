import Head from 'next/head';
import Link from 'next/link';
import {useRouter} from 'next/router';
import {useState, useEffect} from "react";
import {
    Connection,
    Transaction,
} from "@solana/web3.js";
import {createMarket} from "@askbid/client";
import {getProvider} from "../lib/phantom";
import { FontAwesomeIcon } from '@fortawesome/react-fontawesome';
import { faGithub } from '@fortawesome/free-brands-svg-icons';


function WalletButton(props) {
    const [provider, setProvider] = useState(getProvider());
    const [intervalHandle, setIntervalHandle] = useState<number>();
    useEffect(() => {
        if (!provider) {
            const interval = setInterval(() => {
                const provider = getProvider();
                setProvider(provider);
                if (provider) {
                    clearInterval(intervalHandle);
                }
            }, 500);

            // @ts-ignore
            setIntervalHandle(interval);
            return;
        }

        provider.on("connect", () => {
            props.setConnected(true);
        });
        provider.connect({onlyIfTrusted: true});
    }, [provider, props]);

    const onClick = async () => {
        if(provider) {
            await provider.connect();
        }
    }

    return (
        <button
            onClick={onClick}
            className="mr-3 bg-purple-200 border border-purple-300 py-3 px-4 rounded hover:bg-purple-400 hover:border-purple-500">
            Setup a Devnet wallet (Phantom only for now)
        </button>
    );
}

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

export default function Home() {
    const [connection, setConnection] = useState<Connection>(new Connection(process.env.NEXT_PUBLIC_ENDPOINT, 'confirmed'));
    const [isConnected, setConnected] = useState<boolean>(false);
    const [query, setQuery] = useState<string>("");
    const router = useRouter();

    const onQueryChange = event => {
        setQuery(event.target.value);
    };

    const submitSearch = async event => {
        const provider = getProvider();
        const slotOffset = 172800;

        const recentBlockhash = (await connection.getRecentBlockhash()).blockhash;
        const transaction = (new Transaction({recentBlockhash, feePayer: provider.publicKey}));
        const marketAccountKey = await createMarket(provider.publicKey, connection, transaction, provider.publicKey, slotOffset, query);
        const signedTransaction = await provider.signTransaction(transaction);
        const signature = await connection.sendRawTransaction(signedTransaction.serialize());
        await connection.confirmTransaction(signature, 'confirmed');
        await router.push(`/results/${marketAccountKey.publicKey}`);
    };
    return (
        <div>
            <Head>
                <title>🚀 AskBid 🔎</title>
                <link rel="icon" href="/favicon.ico"/>
            </Head>

            <main>
                <div className="flex flex-row-reverse">
                    <a href="https://github.com/Wildebeest/askbid"><FontAwesomeIcon className="text-xs w-8 m-3 opacity-50 hover:opacity-90" icon={faGithub} /></a>
                    <Link href="/docs/index.html"><button className="m-3 hover:text-gray-900 text-gray-500 transition-colors">Documentation</button></Link>
                </div>
                <div className="flex justify-center pt-20">
                    <div>
                        <h1 className="mb-6 text-5xl">🚀 AskBid: A Search Market 🌚</h1>

                        <div className="flex border border-gray-200 rounded p-4 shadow text-xl">
                            <div>🔎</div>
                            <input type="text" className="w-full outline-none px-3" name="query" required autoComplete="off"
                                   onChange={onQueryChange} value={query}/>
                            <div>🇺🇸</div>
                        </div>

                        <div className="mt-8 text-center">
                            {!isConnected && <WalletButton setConnected={setConnected}/>}
                            <SearchButton isConnected={isConnected} setConnected={setConnected} onClick={submitSearch}/>
                        </div>
                    </div>
                </div>
                <hr className="my-12 mx-4" />
                <div className="flex flex-col">
                    <div className="text-center text-xl mb-6 text-2xl">What is a decentralized search market?</div>
                    <div className="container mb-8 mx-auto">
                        <iframe width="888" height="500"
                                src="https://www.youtube.com/embed/MRHTWFL9WPg"
                                frameBorder="0" className="mx-auto" />
                    </div>
                    <div className="flex flex-row ml-4 mr-4 space-x-4">
                        <div className="flex flex-col">
                            <div className="text-center mb-4 text-5xl">🔎</div>
                            You post a search query to the exchange, where many information traders can see it.
                        </div>
                        <div>
                            <div className="text-center mb-4 text-5xl">📈</div>
                            The information traders find results, and bid on whether you will like them.
                        </div>
                        <div>
                            <div className="text-center mb-4 text-5xl">⭐️</div>
                            You visit the results, and mark the one you find most useful.
                        </div>
                        <div>
                            <div className="text-center mb-4 text-5xl">💸</div>
                            The information traders that were right are paid by the ones that were wrong.
                        </div>
                    </div>
                    <div className="text-center text-xl my-12 text-2xl">
                        Want to learn more? Are you a developer? <Link href="/docs/index.html"><a className="hover:text-gray-900 text-gray-500 transition-colors">Check out our docs!</a></Link>
                    </div>
                </div>

            </main>
        </div>
    )
}
