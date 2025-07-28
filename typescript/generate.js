#!/usr/bin/env node

import { compile } from "json-schema-to-typescript";
import fs from "fs";

const jsonSchema = JSON.parse(fs.readFileSync("./schema/schema.json", "utf8"));
const methods = JSON.parse(fs.readFileSync("./schema/methods.json", "utf8"));

let typescriptSource = `

export const AGENT_METHODS = ${JSON.stringify(methods, null, 2)};

${await compile(jsonSchema, "Agent Client Protocol", {
  additionalProperties: false,
  bannerComment: false,
})}
`;

fs.writeFileSync("typescript/acp.ts", typescriptSource, "utf8");
