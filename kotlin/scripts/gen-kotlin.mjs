#!/usr/bin/env node
import fs from "fs/promises";
import path from "path";
import $RefParser from "@apidevtools/json-schema-ref-parser";
import { quicktype, InputData, JSONSchemaInput, FetchingJSONSchemaStore } from "quicktype-core";

// ---- config ----
const PKG = "schema";
const ROOT = path.resolve(process.cwd(), "..");
const SCHEMA = path.join(ROOT, "schema", "schema.json");
const META = path.join(ROOT, "schema", "meta.json");
const OUT_DIR = path.join(process.cwd(), "src/main/kotlin", ...PKG.split("."));
const OUT_SCHEMA = path.join(OUT_DIR, "Schema.kt");
const OUT_META = path.join(OUT_DIR, "Meta.kt");

// ---- load + bundle ----
const bundled = await $RefParser.bundle(SCHEMA);

// ---- strip empty-string keys everywhere (quicktype choke) ----
(function strip(o) {
  if (!o || typeof o !== "object") return;
  if (Object.prototype.hasOwnProperty.call(o, "")) delete o[""];
  for (const k of Object.keys(o)) strip(o[k]);
})(bundled);

// ---- make aggregate root that exposes every $defs entry as a named property ----
const defs = bundled.$defs ?? bundled.definitions ?? {};
const sortedNames = Object.keys(defs).sort();
const properties = Object.fromEntries(sortedNames.map((n) => [n, { $ref: `#/$defs/${n}` }]));
const aggregate = {
  $schema: "https://json-schema.org/draft/2020-12/schema",
  $id: "AcpAggregate",
  type: "object",
  additionalProperties: false,
  properties,
  $defs: defs
};

// ---- quicktype (single source: aggregate keeps $defs context intact) ----
const schemaInput = new JSONSchemaInput(new FetchingJSONSchemaStore());
await schemaInput.addSource({ name: "AcpAggregate", schema: JSON.stringify(aggregate) });

const inputData = new InputData();
inputData.addInput(schemaInput);

const { lines } = await quicktype({
  inputData,
  lang: "kotlin",
  rendererOptions: {
    framework: "kotlinx",
    package: PKG
  },
  inferEnums: true
});

// ---- write Schema.kt ----
await fs.mkdir(OUT_DIR, { recursive: true });
await fs.writeFile(OUT_SCHEMA, lines.join("\n"), "utf8");

// ---- write Meta.kt from meta.json ----
const meta = JSON.parse(await fs.readFile(META, "utf8"));
const toConstBlock = (objName, obj) =>
  `public object ${objName} {\n${Object.keys(obj).sort()
    .map((k) => `  public const val ${k}: String = "${obj[k]}"`)
    .join("\n")}\n}\n`;

const metaKt = `package ${PKG}

${toConstBlock("AgentMethods", meta.agentMethods)}
${toConstBlock("ClientMethods", meta.clientMethods)}
public const val PROTOCOL_VERSION: Int = ${Number(meta.version)}
`;

await fs.writeFile(OUT_META, metaKt, "utf8");

console.log(`Wrote:\n- ${OUT_SCHEMA}\n- ${OUT_META}`);
