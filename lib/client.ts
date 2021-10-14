import {PublicKey} from "@solana/web3.js";
import * as borsh from "borsh";

const PROGRAM_ID = new PublicKey("CtRJbPMscDFRJptvh6snF5GJXDNCJHMFsfYoczds37AV");

class Instruction {
    CreateMarket?: CreateMarket;
    CreateResult?: CreateResult;
    instruction: string;

    constructor(fields: { instruction: string, CreateMarket?: CreateMarket, CreateResult?: CreateResult }) {
        this.CreateMarket = fields.CreateMarket;
        this.CreateResult = fields.CreateResult;
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

const CreateResultSchema = [CreateResult, {
    kind: 'struct',
    fields: [['url', 'string'], ['name', 'string'], ['snippet', 'string'], ['bump_seed', 'u8']]
}]

const InstructionWrapperSchema = [Instruction, {
    kind: 'enum',
    field: 'instruction',
    values: [['CreateMarket', {}], ["CreateResult", {}]]
}];
const CreateMarketSchema = [CreateMarket, {
    kind: 'struct',
    fields: [['expires_slot_offset', 'u64'], ['search_string', 'string']]
}];

// @ts-ignore
const InstructionSchema: borsh.Schema = new Map([
    InstructionWrapperSchema,
    CreateMarketSchema,
    CreateResultSchema]);

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
};
