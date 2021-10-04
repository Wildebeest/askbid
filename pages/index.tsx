import Head from 'next/head';
import {useState, useEffect} from "react";
import {
    PublicKey,
} from "@solana/web3.js";

function SearchButton(props) {
    let hoverStates = "opacity-50";
    if (props.isConnected) {
        hoverStates = "hover:bg-green-400 hover:border-green-500";
    }
    return (
        <button
            className={"mr-3 bg-green-200 border border-green-300 py-3 px-4 rounded " + hoverStates}
            disabled={!props.isConnected}>
            Ask the internet
        </button>
    );
}

interface ConnectOpts {
    onlyIfTrusted: boolean;
}
type PhantomEvent = "disconnect" | "connect";
interface PhantomProvider {
    isConnected: boolean | null;
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
        if (!provider) { return; }

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

export default function Home() {
    const [isConnected, setConnected] = useState<boolean>(false);

    const submitSearch = event => {
        event.preventDefault();

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
                            <input type="text" className="w-full outline-none px-3" name="query" required/>
                            <div>🇺🇸</div>
                        </div>

                        <div className="mt-8 text-center">
                            {!isConnected && <WalletButton setConnected={setConnected} />}
                            <SearchButton isConnected={isConnected} setConnected={setConnected}/>
                        </div>
                    </div>
                </div>

            </main>
        </div>
    )
}
