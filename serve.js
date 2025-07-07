#!/usr/bin/env node

const http = require("http");
const fs = require("fs");
const path = require("path");
const process = require("process");

const PORT = 8080;

const mimeTypes = {
  ".html": "text/html",
  ".js": "application/javascript",
  ".wasm": "application/wasm",
  ".json": "application/json",
  ".css": "text/css",
  ".ts": "application/typescript",
};

const server = http.createServer((req, res) => {
  // Handle root path
  let filePath = req.url === "/" ? "/demo.html" : req.url;
  filePath = path.join(__dirname, filePath);

  const ext = path.extname(filePath);
  const mimeType = mimeTypes[ext] || "text/plain";

  fs.readFile(filePath, (err, data) => {
    if (err) {
      res.writeHead(404, { "Content-Type": "text/plain" });
      res.end(`File not found: ${req.url}`);
      console.log(`404: ${req.url}`);
    } else {
      res.writeHead(200, {
        "Content-Type": mimeType,
        "Cross-Origin-Embedder-Policy": "require-corp",
        "Cross-Origin-Opener-Policy": "same-origin",
      });
      res.end(data);
      console.log(`200: ${req.url} (${mimeType})`);
    }
  });
});

server.listen(PORT, () => {
  console.log(`🌐 Development server running at http://localhost:${PORT}`);
  console.log(`📁 Serving files from: ${__dirname}`);
  console.log(`🎲 Open http://localhost:${PORT} to view the TBL Parser demo`);
  console.log("\nPress Ctrl+C to stop the server");
});
