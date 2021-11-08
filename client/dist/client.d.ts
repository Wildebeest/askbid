import { Connection, Keypair, PublicKey, Transaction, TransactionInstruction } from "@solana/web3.js";
import * as borsh from "borsh";
import BN from "bn.js";
declare const PROGRAM_ID: PublicKey;
declare const LAMPORTS_PER_TOKEN = 100000;
declare class Instruction {
    CreateMarket?: CreateMarket;
    CreateResult?: CreateResult;
    Deposit?: Deposit;
    Withdraw?: Withdraw;
    Decide?: Decide;
    CreateOrder?: CreateOrder;
    instruction: string;
    constructor(fields: {
        instruction: string;
        CreateMarket?: CreateMarket;
        CreateResult?: CreateResult;
        Deposit?: Deposit;
        CreateOrder?: CreateOrder;
        Decide?: Decide;
    });
}
declare class CreateMarket {
    expires_slot_offset: number;
    search_string: string;
    constructor(expires_slot_offset: number, search_string: string);
}
declare class CreateResult {
    url: string;
    name: string;
    snippet: string;
    bump_seed: number;
    constructor(url: string, name: string, snippet: string, bump_seed: number);
}
declare class Deposit {
    amount: number;
    constructor(amount: number);
}
declare class Withdraw {
    amount: number;
    constructor(amount: number);
}
declare class Decide {
}
declare class CreateOrder {
    side: number;
    price: number;
    quantity: number;
    escrow_bump_seed: number;
    constructor(side: number, price: number, quantity: number, escrow_bump_seed: number);
}
declare class Order {
    account_type: number;
    account_version: number;
    search_market: Uint8Array;
    result: Uint8Array;
    sol_account: Uint8Array;
    token_account: Uint8Array;
    side: number;
    price: BN;
    quantity: BN;
    escrow_bump_seed: number;
    creation_slot: number;
    execution_authority: Uint8Array;
    constructor(fields: {
        search_market: Uint8Array;
        result: Uint8Array;
        sol_account: Uint8Array;
        token_account: Uint8Array;
        side: number;
        price: number;
        quantity: number;
        escrow_bump_seed: number;
        creation_slot: number;
        execution_authority: Uint8Array;
    });
}
declare const OrderSchema: borsh.Schema;
declare const CreateOrderSchema: (typeof CreateOrder | {
    kind: string;
    fields: string[][];
})[];
declare const DepositSchema: (typeof Deposit | {
    kind: string;
    fields: string[][];
})[];
declare const WithdrawSchema: (typeof Withdraw | {
    kind: string;
    fields: string[][];
})[];
declare const DecideSchema: (typeof Decide | {
    kind: string;
    fields: never[];
})[];
declare const CreateResultSchema: (typeof CreateResult | {
    kind: string;
    fields: string[][];
})[];
declare const InstructionWrapperSchema: (typeof Instruction | {
    kind: string;
    field: string;
    values: {}[][];
})[];
declare const CreateMarketSchema: (typeof CreateMarket | {
    kind: string;
    fields: string[][];
})[];
declare const InstructionSchema: borsh.Schema;
declare class SearchMarketAccount {
    account_type: number;
    account_version: number;
    decision_authority: Uint8Array;
    search_string: string;
    best_result: Uint8Array;
    expires_slot: number;
    constructor(fields: {
        decision_authority: Uint8Array;
        search_string: string;
        best_result: Uint8Array;
        expires_slot: number;
    });
}
declare const SearchMarketAccountSchema: borsh.Schema;
declare class ResultAccount {
    account_type: number;
    account_version: number;
    search_market: Uint8Array;
    url: string;
    name: string;
    snippet: string;
    yes_mint: Uint8Array;
    no_mint: Uint8Array;
    bump_seed: number;
    constructor(fields: {
        search_market: Uint8Array;
        url: string;
        name: string;
        snippet: string;
        yes_mint: Uint8Array;
        no_mint: Uint8Array;
        bump_seed: number;
    });
}
declare const ResultAccountSchema: borsh.Schema;
declare function createMarketInstruction(market: PublicKey, decisionAuthority: PublicKey, slotOffset: number, query: string): TransactionInstruction;
declare function createMarket(payer: PublicKey, connection: Connection, transaction: Transaction, decisionAuthority: PublicKey, slotOffset: number, query: string): Promise<Keypair>;
export { PROGRAM_ID, Instruction, InstructionSchema, InstructionWrapperSchema, CreateMarketSchema, CreateMarket, CreateResultSchema, createMarketInstruction, createMarket, CreateResult, SearchMarketAccountSchema, SearchMarketAccount, ResultAccount, ResultAccountSchema, Deposit, DepositSchema, Withdraw, WithdrawSchema, Decide, DecideSchema, CreateOrder, CreateOrderSchema, Order, OrderSchema, LAMPORTS_PER_TOKEN, };
