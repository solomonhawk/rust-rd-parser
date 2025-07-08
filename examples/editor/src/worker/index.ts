export const workerInstance = new ComlinkWorker<typeof import("./worker.js")>(
  new URL("./worker", import.meta.url),
  {
    name: "wasm-bridge-worker",
    type: "module",
  }
);
