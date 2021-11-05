import {PublicKey} from "@solana/web3.js";
import * as borsh from "borsh";
import BN from "bn.js";

const PROGRAM_ID = new PublicKey("CtRJbPMscDFRJptvh6snF5GJXDNCJHMFsfYoczds37AV");
const LAMPORTS_PER_TOKEN = 100000;

class Instruction {
    CreateMarket?: CreateMarket;
    CreateResult?: CreateResult;
    Deposit?: Deposit;
    Withdraw?: Withdraw;
    Decide?: Decide;
    CreateOrder?: CreateOrder;
    instruction: string;

    constructor(fields: { instruction: string, CreateMarket?: CreateMarket, CreateResult?: CreateResult, Deposit?: Deposit, CreateOrder?: CreateOrder, Decide?: Decide }) {
        this.CreateMarket = fields.CreateMarket;
        this.CreateResult = fields.CreateResult;
        this.Deposit = fields.Deposit;
        this.Decide = fields.Decide;
        this.CreateOrder = fields.CreateOrder;
        this.instruction = fields.instruction;
    }
}

class CreateMarket {
    expires_slot_offset: number;
    search_string: string;

    constructor(expires_slot_offset: number, search_string: string) {
        this.expires_slot_offset = expires_slot_offset;
        this.search_string = search_string;
    }
}

class CreateResult {
    url: string;
    name: string;
    snippet: string;
    bump_seed: number;

    constructor(url: string,
                name: string,
                snippet: string,
                bump_seed: number,) {
        this.url = url;
        this.name = name;
        this.snippet = snippet;
        this.bump_seed = bump_seed;
    }
}

class Deposit {
    amount: number;

    constructor(amount: number) {
        this.amount = amount;
    }
}

class Withdraw {
    amount: number;

    constructor(amount: number) {
        this.amount = amount;
    }
}

class Decide {

}

class CreateOrder {
    side: number;
    price: number;
    quantity: number;
    escrow_bump_seed: number;

    constructor(side: number, price: number, quantity: number, escrow_bump_seed: number) {
        this.side = side;
        this.price = price;
        this.quantity = quantity;
        this.escrow_bump_seed = escrow_bump_seed;
    }
}

class Order {
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
        search_market: Uint8Array,
        result: Uint8Array,
        sol_account: Uint8Array,
        token_account: Uint8Array,
        side: number,
        price: number,
        quantity: number,
        escrow_bump_seed: number,
        creation_slot: number,
        execution_authority: Uint8Array,
    }) {
        this.account_type = 2;
        this.account_version = 0;
        this.search_market = fields.search_market;
        this.result = fields.result;
        this.sol_account = fields.sol_account;
        this.token_account = fields.token_account;
        this.side = fields.side;
        this.price = new BN(fields.price);
        this.quantity = new BN(fields.quantity);
        this.escrow_bump_seed = fields.escrow_bump_seed;
        this.creation_slot = fields.creation_slot;
        this.execution_authority = fields.execution_authority;
    }
}

// @ts-ignore
const OrderSchema: borsh.Schema = new Map([[Order, {
    kind: 'struct',
    fields: [
        ['account_type', 'u8'],
        ['account_version', 'u8'],
        ['search_market', [32]],
        ['result', [32]],
        ['sol_account', [32]],
        ['token_account', [32]],
        ['side', 'u8'],
        ['price', 'u64'],
        ['quantity', 'u64'],
        ['escrow_bump_seed', 'u8'],
        ['creation_slot', 'u64'],
        ['execution_authority', [32]],
    ]
}]]);

const CreateOrderSchema = [CreateOrder, {
    kind: 'struct',
    fields: [['side', 'u8'], ['price', 'u64'], ['quantity', 'u64'], ['escrow_bump_seed', 'u8']]
}];

const DepositSchema = [Deposit, {
    kind: 'struct',
    fields: [['amount', 'u64']]
}];

const WithdrawSchema = [Withdraw, {
    kind: 'struct',
    fields: [['amount', 'u64']]
}];

const DecideSchema = [Decide, {
    kind: 'struct',
    fields: [],
}]


const CreateResultSchema = [CreateResult, {
    kind: 'struct',
    fields: [['url', 'string'], ['name', 'string'], ['snippet', 'string'], ['bump_seed', 'u8']]
}]

const InstructionWrapperSchema = [Instruction, {
    kind: 'enum',
    field: 'instruction',
    values: [['CreateMarket', {}], ["CreateResult", {}], ["Deposit", {}], ["Withdraw", {}], ["Decide", {}], ["CreateOrder", {}]]
}];
const CreateMarketSchema = [CreateMarket, {
    kind: 'struct',
    fields: [['expires_slot_offset', 'u64'], ['search_string', 'string']]
}];

// @ts-ignore
const InstructionSchema: borsh.Schema = new Map([
    InstructionWrapperSchema,
    CreateMarketSchema,
    CreateResultSchema,
    DepositSchema,
    WithdrawSchema,
    DecideSchema,
    CreateOrderSchema]);

class SearchMarketAccount {
    account_type: number;
    account_version: number;
    decision_authority: Uint8Array;
    search_string: string;
    best_result: Uint8Array;
    expires_slot: number;

    constructor(fields: { decision_authority: Uint8Array, search_string: string, best_result: Uint8Array, expires_slot: number }) {
        this.account_type = 0;
        this.account_version = 0;
        this.decision_authority = fields.decision_authority;
        this.search_string = fields.search_string;
        this.best_result = fields.best_result;
        this.expires_slot = fields.expires_slot;
    }
}

const SearchMarketAccountSchema: borsh.Schema = new Map([[SearchMarketAccount, {
    kind: 'struct',
    fields: [
        ['account_type', 'u8'],
        ['account_version', 'u8'],
        ['decision_authority', [32]],
        ['search_string', 'string'],
        ['best_result', [32]],
        ['expires_slot', 'u64']],
}]]);

class ResultAccount {
    account_type: number;
    account_version: number;
    search_market: Uint8Array;
    url: string;
    name: string;
    snippet: string;
    yes_mint: Uint8Array;
    no_mint: Uint8Array;
    bump_seed: number;

    constructor(fields: { search_market: Uint8Array, url: string, name: string, snippet: string, yes_mint: Uint8Array, no_mint: Uint8Array, bump_seed: number }) {
        this.account_type = 1;
        this.account_version = 0;
        this.search_market = fields.search_market;
        this.url = fields.url;
        this.name = fields.name;
        this.snippet = fields.snippet;
        this.yes_mint = fields.yes_mint;
        this.no_mint = fields.no_mint;
        this.bump_seed = fields.bump_seed;
    }
}

const ResultAccountSchema: borsh.Schema = new Map([[ResultAccount, {
    kind: 'struct',
    fields: [
        ['account_type', 'u8'],
        ['account_version', 'u8'],
        ['search_market', [32]],
        ['url', 'string'],
        ['name', 'string'],
        ['snippet', 'string'],
        ['yes_mint', [32]],
        ['no_mint', [32]],
        ['bump_seed', 'u8'],
    ]
}]]);

export {
    PROGRAM_ID,
    Instruction,
    InstructionSchema,
    InstructionWrapperSchema,
    CreateMarketSchema,
    CreateMarket,
    CreateResultSchema,
    CreateResult,
    SearchMarketAccountSchema,
    SearchMarketAccount,
    ResultAccount,
    ResultAccountSchema,
    Deposit,
    DepositSchema,
    Withdraw,
    WithdrawSchema,
    Decide,
    DecideSchema,
    CreateOrder,
    CreateOrderSchema,
    Order,
    OrderSchema,
    LAMPORTS_PER_TOKEN,
};
