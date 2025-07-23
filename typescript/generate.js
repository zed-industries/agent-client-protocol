#!/usr/bin/env node

// import { execFileSync } from "child_process"
import { compile } from "json-schema-to-typescript";
import fs from "fs";

const jsonSchema = JSON.parse(fs.readFileSync("./schema.json", "utf8"));

let typescriptSource = `

export const NEW_SESSION_TOOL_NAME = "acp__new_session";
export const LOAD_SESSION_TOOL_NAME = "acp__load_session";
export const PROMPT_TOOL_NAME = "acp__prompt";

${await compile(jsonSchema, "Agent Coding Protocol", {
  additionalProperties: false,
  bannerComment: false,
})}
`;

fs.writeFileSync("typescript/acp.ts", typescriptSource, "utf8");
