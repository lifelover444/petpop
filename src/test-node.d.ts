declare module "node:fs" {
  export function readFileSync(path: URL, encoding: "utf-8"): string;
}
