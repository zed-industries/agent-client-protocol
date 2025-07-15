#!/usr/bin/env node

// import { execFileSync } from "child_process"
import { compile } from "json-schema-to-typescript";
import fs from "fs";

// execFileSync("cargo", ["run"])

const jsonSchema = JSON.parse(fs.readFileSync("./schema.json", "utf8"));
const clientMethods = JSON.parse(
  fs.readFileSync("./target/client_requests.json", "utf8"),
);
const agentMethods = JSON.parse(
  fs.readFileSync("./target/agent_requests.json", "utf8"),
);

let typescriptSource = `import pkg from "../package.json" with { type: "json" };

export const LATEST_PROTOCOL_VERSION = pkg.version;

${await compile(jsonSchema, "Agent Coding Protocol", {
  additionalProperties: false,
  bannerComment: false,
})}

${requestMapToInterface("Client", clientMethods)}

${requestMapToInterface("Agent", agentMethods)}
`;

fs.writeFileSync("typescript/schema.ts", typescriptSource, "utf8");

function requestMapToInterface(name, methods) {
  let code = `export abstract class ${name} {\n`;

  for (const {
    name,
    requestType,
    responseType,
    paramPayload,
    responsePayload,
  } of methods) {
    code += `abstract ${name}`;
    if (paramPayload) {
      code += `(params: ${requestType})`;
    } else {
      code += `()`;
    }
    if (responsePayload) {
      code += `: Promise<${responseType}>;\n\n`;
    } else {
      code += `: Promise<void>;\n\n`;
    }
  }
  code += "}\n\n";

  code += `export const ${name.toUpperCase()}_METHODS = {\n`;
  for (const { name } of methods) {
    code += `  ${name.replace(/([a-z])([A-Z])/g, "$1_$2").toUpperCase()}: "${name}",\n`;
  }
  code += `};\n`;

  return code;
}
