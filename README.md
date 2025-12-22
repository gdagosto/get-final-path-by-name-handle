# @gdagosto/get-final-path-by-name-handle

Native Node.js addon that exposes Windows' GetFinalPathNameByHandle functionality.

Important: This package is Windows-only. Prebuilt binaries are published as release assets; the installer will try to download a matching prebuilt binary and fall back to building from source (requires Rust + MSVC).

Install:

```bash
# will attempt to download prebuilt binary for your Node ABI
npm install @gdagosto/get-final-path-by-name-handle
```

API:

```ts
import { getFinalPathByNameHandle } from '@gdagosto/get-final-path-by-name-handle';

console.log(getFinalPathByNameHandle(someHandle));
```

Building locally (Windows):

```powershell
pnpm install
pnpm run build
pnpm run build:rust
```
