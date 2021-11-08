"use strict";
var __createBinding = (this && this.__createBinding) || (Object.create ? (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    Object.defineProperty(o, k2, { enumerable: true, get: function() { return m[k]; } });
}) : (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    o[k2] = m[k];
}));
var __setModuleDefault = (this && this.__setModuleDefault) || (Object.create ? (function(o, v) {
    Object.defineProperty(o, "default", { enumerable: true, value: v });
}) : function(o, v) {
    o["default"] = v;
});
var __importStar = (this && this.__importStar) || function (mod) {
    if (mod && mod.__esModule) return mod;
    var result = {};
    if (mod != null) for (var k in mod) if (k !== "default" && Object.prototype.hasOwnProperty.call(mod, k)) __createBinding(result, mod, k);
    __setModuleDefault(result, mod);
    return result;
};
var __awaiter = (this && this.__awaiter) || function (thisArg, _arguments, P, generator) {
    function adopt(value) { return value instanceof P ? value : new P(function (resolve) { resolve(value); }); }
    return new (P || (P = Promise))(function (resolve, reject) {
        function fulfilled(value) { try { step(generator.next(value)); } catch (e) { reject(e); } }
        function rejected(value) { try { step(generator["throw"](value)); } catch (e) { reject(e); } }
        function step(result) { result.done ? resolve(result.value) : adopt(result.value).then(fulfilled, rejected); }
        step((generator = generator.apply(thisArg, _arguments || [])).next());
    });
};
var __generator = (this && this.__generator) || function (thisArg, body) {
    var _ = { label: 0, sent: function() { if (t[0] & 1) throw t[1]; return t[1]; }, trys: [], ops: [] }, f, y, t, g;
    return g = { next: verb(0), "throw": verb(1), "return": verb(2) }, typeof Symbol === "function" && (g[Symbol.iterator] = function() { return this; }), g;
    function verb(n) { return function (v) { return step([n, v]); }; }
    function step(op) {
        if (f) throw new TypeError("Generator is already executing.");
        while (_) try {
            if (f = 1, y && (t = op[0] & 2 ? y["return"] : op[0] ? y["throw"] || ((t = y["return"]) && t.call(y), 0) : y.next) && !(t = t.call(y, op[1])).done) return t;
            if (y = 0, t) op = [op[0] & 2, t.value];
            switch (op[0]) {
                case 0: case 1: t = op; break;
                case 4: _.label++; return { value: op[1], done: false };
                case 5: _.label++; y = op[1]; op = [0]; continue;
                case 7: op = _.ops.pop(); _.trys.pop(); continue;
                default:
                    if (!(t = _.trys, t = t.length > 0 && t[t.length - 1]) && (op[0] === 6 || op[0] === 2)) { _ = 0; continue; }
                    if (op[0] === 3 && (!t || (op[1] > t[0] && op[1] < t[3]))) { _.label = op[1]; break; }
                    if (op[0] === 6 && _.label < t[1]) { _.label = t[1]; t = op; break; }
                    if (t && _.label < t[2]) { _.label = t[2]; _.ops.push(op); break; }
                    if (t[2]) _.ops.pop();
                    _.trys.pop(); continue;
            }
            op = body.call(thisArg, _);
        } catch (e) { op = [6, e]; y = 0; } finally { f = t = 0; }
        if (op[0] & 5) throw op[1]; return { value: op[0] ? op[1] : void 0, done: true };
    }
};
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.LAMPORTS_PER_TOKEN = exports.OrderSchema = exports.Order = exports.CreateOrderSchema = exports.CreateOrder = exports.DecideSchema = exports.Decide = exports.WithdrawSchema = exports.Withdraw = exports.DepositSchema = exports.Deposit = exports.ResultAccountSchema = exports.ResultAccount = exports.SearchMarketAccount = exports.SearchMarketAccountSchema = exports.CreateResult = exports.createMarket = exports.createMarketInstruction = exports.CreateResultSchema = exports.CreateMarket = exports.CreateMarketSchema = exports.InstructionWrapperSchema = exports.InstructionSchema = exports.Instruction = exports.PROGRAM_ID = void 0;
var web3_js_1 = require("@solana/web3.js");
var borsh = __importStar(require("borsh"));
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
function createMarketInstruction(market, decisionAuthority, slotOffset, query) {
    var data = borsh.serialize(InstructionSchema, new Instruction({
        instruction: "CreateMarket",
        CreateMarket: new CreateMarket(slotOffset, query),
    }));
    return new web3_js_1.TransactionInstruction({
        keys: [
            {
                pubkey: market,
                isSigner: false,
                isWritable: true,
            },
            {
                pubkey: decisionAuthority,
                isSigner: true,
                isWritable: false
            }
        ], programId: PROGRAM_ID, data: Buffer.from(data)
    });
}
exports.createMarketInstruction = createMarketInstruction;
function createMarket(payer, connection, transaction, decisionAuthority, slotOffset, query) {
    return __awaiter(this, void 0, void 0, function () {
        var searchMarketAccount, accountSize, rentExemptAmount, marketAccountKey, newAccountInstruction, transactionInstruction;
        return __generator(this, function (_a) {
            switch (_a.label) {
                case 0:
                    searchMarketAccount = new SearchMarketAccount({
                        decision_authority: decisionAuthority.toBytes(),
                        search_string: query,
                        expires_slot: 0,
                        best_result: web3_js_1.PublicKey.default.toBytes()
                    });
                    accountSize = borsh.serialize(SearchMarketAccountSchema, searchMarketAccount).byteLength;
                    return [4 /*yield*/, connection.getMinimumBalanceForRentExemption(accountSize)];
                case 1:
                    rentExemptAmount = _a.sent();
                    marketAccountKey = web3_js_1.Keypair.generate();
                    newAccountInstruction = web3_js_1.SystemProgram.createAccount({
                        fromPubkey: payer,
                        programId: PROGRAM_ID,
                        newAccountPubkey: marketAccountKey.publicKey,
                        lamports: rentExemptAmount,
                        space: accountSize
                    });
                    transactionInstruction = createMarketInstruction(marketAccountKey.publicKey, decisionAuthority, slotOffset, query);
                    transaction.add(newAccountInstruction)
                        .add(transactionInstruction);
                    transaction.partialSign(marketAccountKey);
                    return [2 /*return*/, marketAccountKey];
            }
        });
    });
}
exports.createMarket = createMarket;
