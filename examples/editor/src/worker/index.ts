export const workerInstance = new ComlinkWorker<typeof import("./worker.ts")>(
  new URL("./worker", import.meta.url),
  {
    name: "wasm-bridge-worker",
    type: "module",
  }
);
