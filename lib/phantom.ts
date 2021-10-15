import {PublicKey, Transaction} from "@solana/web3.js";

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

export function getProvider(): PhantomProvider | undefined {
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
}