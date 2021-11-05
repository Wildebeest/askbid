"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.LAMPORTS_PER_TOKEN = exports.OrderSchema = exports.Order = exports.CreateOrderSchema = exports.CreateOrder = exports.DecideSchema = exports.Decide = exports.WithdrawSchema = exports.Withdraw = exports.DepositSchema = exports.Deposit = exports.ResultAccountSchema = exports.ResultAccount = exports.SearchMarketAccount = exports.SearchMarketAccountSchema = exports.CreateResult = exports.CreateResultSchema = exports.CreateMarket = exports.CreateMarketSchema = exports.InstructionWrapperSchema = exports.InstructionSchema = exports.Instruction = exports.PROGRAM_ID = void 0;
var web3_js_1 = require("@solana/web3.js");
var bn_js_1 = __importDefault(require("bn.js"));
var PROGRAM_ID = new web3_js_1.PublicKey("CtRJbPMscDFRJptvh6snF5GJXDNCJHMFsfYoczds37AV");
exports.PROGRAM_ID = PROGRAM_ID;
var LAMPORTS_PER_TOKEN = 100000;
exports.LAMPORTS_PER_TOKEN = LAMPORTS_PER_TOKEN;
var Instruction = /** @class */ (function () {
    function Instruction(fields) {
        this.CreateMarket = fields.CreateMarket;
        this.CreateResult = fields.CreateResult;
        this.Deposit = fields.Deposit;
        this.Decide = fields.Decide;
        this.CreateOrder = fields.CreateOrder;
        this.instruction = fields.instruction;
    }
    return Instruction;
}());
exports.Instruction = Instruction;
var CreateMarket = /** @class */ (function () {
    function CreateMarket(expires_slot_offset, search_string) {
        this.expires_slot_offset = expires_slot_offset;
        this.search_string = search_string;
    }
    return CreateMarket;
}());
exports.CreateMarket = CreateMarket;
var CreateResult = /** @class */ (function () {
    function CreateResult(url, name, snippet, bump_seed) {
        this.url = url;
        this.name = name;
        this.snippet = snippet;
        this.bump_seed = bump_seed;
    }
    return CreateResult;
}());
exports.CreateResult = CreateResult;
var Deposit = /** @class */ (function () {
    function Deposit(amount) {
        this.amount = amount;
    }
    return Deposit;
}());
exports.Deposit = Deposit;
var Withdraw = /** @class */ (function () {
    function Withdraw(amount) {
        this.amount = amount;
    }
    return Withdraw;
}());
exports.Withdraw = Withdraw;
var Decide = /** @class */ (function () {
    function Decide() {
    }
    return Decide;
}());
exports.Decide = Decide;
var CreateOrder = /** @class */ (function () {
    function CreateOrder(side, price, quantity, escrow_bump_seed) {
        this.side = side;
        this.price = price;
        this.quantity = quantity;
        this.escrow_bump_seed = escrow_bump_seed;
    }
    return CreateOrder;
}());
exports.CreateOrder = CreateOrder;
var Order = /** @class */ (function () {
    function Order(fields) {
        this.account_type = 2;
        this.account_version = 0;
        this.search_market = fields.search_market;
        this.result = fields.result;
        this.sol_account = fields.sol_account;
        this.token_account = fields.token_account;
        this.side = fields.side;
        this.price = new bn_js_1.default(fields.price);
        this.quantity = new bn_js_1.default(fields.quantity);
        this.escrow_bump_seed = fields.escrow_bump_seed;
        this.creation_slot = fields.creation_slot;
        this.execution_authority = fields.execution_authority;
    }
    return Order;
}());
exports.Order = Order;
// @ts-ignore
var OrderSchema = new Map([[Order, {
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
exports.OrderSchema = OrderSchema;
var CreateOrderSchema = [CreateOrder, {
        kind: 'struct',
        fields: [['side', 'u8'], ['price', 'u64'], ['quantity', 'u64'], ['escrow_bump_seed', 'u8']]
    }];
exports.CreateOrderSchema = CreateOrderSchema;
var DepositSchema = [Deposit, {
        kind: 'struct',
        fields: [['amount', 'u64']]
    }];
exports.DepositSchema = DepositSchema;
var WithdrawSchema = [Withdraw, {
        kind: 'struct',
        fields: [['amount', 'u64']]
    }];
exports.WithdrawSchema = WithdrawSchema;
var DecideSchema = [Decide, {
        kind: 'struct',
        fields: [],
    }];
exports.DecideSchema = DecideSchema;
var CreateResultSchema = [CreateResult, {
        kind: 'struct',
        fields: [['url', 'string'], ['name', 'string'], ['snippet', 'string'], ['bump_seed', 'u8']]
    }];
exports.CreateResultSchema = CreateResultSchema;
var InstructionWrapperSchema = [Instruction, {
        kind: 'enum',
        field: 'instruction',
        values: [['CreateMarket', {}], ["CreateResult", {}], ["Deposit", {}], ["Withdraw", {}], ["Decide", {}], ["CreateOrder", {}]]
    }];
exports.InstructionWrapperSchema = InstructionWrapperSchema;
var CreateMarketSchema = [CreateMarket, {
        kind: 'struct',
        fields: [['expires_slot_offset', 'u64'], ['search_string', 'string']]
    }];
exports.CreateMarketSchema = CreateMarketSchema;
// @ts-ignore
var InstructionSchema = new Map([
    InstructionWrapperSchema,
    CreateMarketSchema,
    CreateResultSchema,
    DepositSchema,
    WithdrawSchema,
    DecideSchema,
    CreateOrderSchema
]);
exports.InstructionSchema = InstructionSchema;
var SearchMarketAccount = /** @class */ (function () {
    function SearchMarketAccount(fields) {
        this.account_type = 0;
        this.account_version = 0;
        this.decision_authority = fields.decision_authority;
        this.search_string = fields.search_string;
        this.best_result = fields.best_result;
        this.expires_slot = fields.expires_slot;
    }
    return SearchMarketAccount;
}());
exports.SearchMarketAccount = SearchMarketAccount;
var SearchMarketAccountSchema = new Map([[SearchMarketAccount, {
            kind: 'struct',
            fields: [
                ['account_type', 'u8'],
                ['account_version', 'u8'],
                ['decision_authority', [32]],
                ['search_string', 'string'],
                ['best_result', [32]],
                ['expires_slot', 'u64']
            ],
        }]]);
exports.SearchMarketAccountSchema = SearchMarketAccountSchema;
var ResultAccount = /** @class */ (function () {
    function ResultAccount(fields) {
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
    return ResultAccount;
}());
exports.ResultAccount = ResultAccount;
var ResultAccountSchema = new Map([[ResultAccount, {
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
exports.ResultAccountSchema = ResultAccountSchema;
