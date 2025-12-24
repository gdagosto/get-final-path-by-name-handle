import { createRequire } from "module";
const require = createRequire(import.meta.url);

let addon: any;
try {
  addon = require("./index.node");
} catch (err) {
  addon = require("../index.node");
}

export function getFinalPathByNameHandle(nameHandle: string): string {
  return addon.getFinalPathNameByHandle(nameHandle);
}
