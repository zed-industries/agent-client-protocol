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

let typescriptSource = `import semver from 'semver';
import pkg from "../package.json" with { type: "json" };

export const LATEST_PROTOCOL_VERSION = pkg.version;

${await compile(jsonSchema, "Agent Coding Protocol", {
  additionalProperties: false,
  bannerComment: false,
})}

export interface Method {
  name: string;
  requestType: string;
  paramPayload: boolean;
  responseType: string;
  responsePayload: boolean;
}

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
  code += `
  /**
   * Validates that the provided version is compatible with the current protocol version.
   *
   * @param version - The version string to validate
   * @throws {Error} If the version is not compatible with the current protocol version
   */
  validateVersion(version: string) {
    if (!semver.satisfies(LATEST_PROTOCOL_VERSION, \`^\${version}\`)) {
      throw new Error(\`Incompatible versions: Requested \${version} / Supported: ^\${LATEST_PROTOCOL_VERSION}\`);
    }
  }\n`;
  code += "}\n\n";

  code += `export const ${name.toUpperCase()}_METHODS: Method[] = ${JSON.stringify(methods, null, 2)};`;

  return code;
}
