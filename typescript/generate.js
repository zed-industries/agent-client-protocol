#!/usr/bin/env node

import { compile } from "json-schema-to-typescript";
import { generate } from "ts-to-zod";
import fs from "fs";

const jsonSchema = JSON.parse(fs.readFileSync("./schema/schema.json", "utf8"));
const methods = JSON.parse(fs.readFileSync("./schema/methods.json", "utf8"));

const tsSrc = await compile(jsonSchema, "Agent Client Protocol", {
  additionalProperties: false,
  bannerComment: false,
});

const zodGenerator = generate({ sourceText: tsSrc });
const zodSchemas = zodGenerator.getZodSchemasFile();
const zodInfer = zodGenerator.getInferredTypes("./zod");

const acpTs = `
${zodInfer}

export const AGENT_METHODS = ${JSON.stringify(methods, null, 2)};
`;

fs.writeFileSync("typescript/acp.ts", acpTs, "utf8");

const schemasTs = `
${zodSchemas}
`;

fs.writeFileSync("typescript/zod.ts", schemasTs, "utf8");
