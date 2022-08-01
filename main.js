import mod from "module";
import { name } from "mod\\u1011";
export var variable_5 = 5;

export function function_1() {}
export let let_6 = 6;
export * from "external";
import * as ns from "external2";
export { /*pure*/ pured } from "module";
export { a as b } from "external3";
export { ns };
export { named, named2 } from "module";
export { another, another2 } from "internal";
export { another1 } from "internal";
export { ns1, ns2, ns3 };
export { name as out };
export * from "ex";
export * as su from "mod";
export default su;
export class Hello {}
