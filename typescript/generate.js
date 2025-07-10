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

let typescriptSource = await compile(jsonSchema, "Agent Coding Protocol", {
  additionalProperties: false,
  bannerComment: false,
});

const clientInterface = requestMapToInterface("Client", clientMethods);
const agentInterface = requestMapToInterface("Agent", agentMethods);

typescriptSource += "\n\nexport interface Method {\n";
typescriptSource += "  name: string;\n";
typescriptSource += "  requestType: string;\n";
typescriptSource += "  paramPayload: boolean;\n";
typescriptSource += "  responseType: string;\n";
typescriptSource += "  responsePayload: boolean;\n";
typescriptSource += "  errorType: string;\n";
typescriptSource += "}\n";

typescriptSource +=
  "\nexport type Result<T, E = Error> = { ok: T } | { error: E };\n";

typescriptSource +=
  "\nexport type VoidResult<E = Error> = void | { error: E };\n";

typescriptSource += "\n" + clientInterface + "\n\n" + agentInterface + "\n";

fs.writeFileSync("typescript/schema.ts", typescriptSource, "utf8");

function requestMapToInterface(name, methods) {
  let code = `export interface ${name} {\n`;

  for (const {
    name,
    requestType,
    responseType,
    paramPayload,
    responsePayload,
    errorType,
  } of methods) {
    code += name;
    if (paramPayload) {
      code += `(params: ${requestType})`;
    } else {
      code += `()`;
    }
    code += `: Promise<`;
    if (responsePayload) {
      code += `Result<${responseType}, ${errorType}>`;
    } else {
      code += `VoidResult<${errorType}>`;
    }
    code += `>;\n`;
  }
  code += "}\n\n";

  code += `export const ${name.toUpperCase()}_METHODS: Method[] = ${JSON.stringify(methods, null, 2)};`;

  return code;
}
