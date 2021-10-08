import {PublicKey} from "@solana/web3.js";
import * as borsh from "borsh";

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

// @ts-ignore
const InstructionSchema: borsh.Schema = new Map([
    InstructionWrapperSchema,
    CreateMarketSchema]);

class SearchMarketAccount {
    decision_authority: Uint8Array;
    search_string: string;
    best_result: Uint8Array;
    expires_slot: number;

    constructor(fields: { decision_authority: Uint8Array, search_string: string, best_result: Uint8Array, expires_slot: number }) {
        this.decision_authority = fields.decision_authority;
        this.search_string = fields.search_string;
        this.best_result = fields.best_result;
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

class ResultAccount {
    search_market: PublicKey;
    url: string;
    name: string;
    snippet: string;

    constructor(fields: { search_market: Uint8Array, url: string, name: string, snippet: string, yes_mint: Uint8Array, no_mint: Uint8Array, bump_seed: number }) {
        this.search_market = new PublicKey(fields.search_market);
        this.url = fields.url;
        this.name = fields.name;
        this.snippet = fields.snippet;
    }
}

const ResultAccountSchema: borsh.Schema = new Map([[ResultAccount, {
    kind: 'struct',
    fields: [
        [
            ['search_market', [32]],
            ['url', 'string'],
            ['name', 'string'],
            ['snippet', 'string'],
            ['yes_mint', [32]],
            ['no_mint', [32]],
            ['bump_seed', 'u8'],
        ]
    ]
}]]);

export {
    PROGRAM_ID,
    Instruction,
    InstructionSchema,
    InstructionWrapperSchema,
    CreateMarketSchema,
    CreateMarket,
    SearchMarketAccountSchema,
    SearchMarketAccount,
    ResultAccount,
    ResultAccountSchema,
};
