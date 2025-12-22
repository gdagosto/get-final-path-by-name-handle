import { createRequire } from "module";
const require = createRequire(import.meta.url);

const addon = require("./index.node");

export function getFinalPathByNameHandle(nameHandle: number): string {
  return addon.getFinalPathByNameHandle(nameHandle);
}
