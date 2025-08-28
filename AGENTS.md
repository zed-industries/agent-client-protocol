All paths in the protocol should be absolute

## Adding new methods

- Create empty params and output structs in rust/client.rs or rust/agent.rs under the corresponding section. I'll add the fields myself.
- If the protocol method name is `noun/verb`, use `verb_noun` for the user facing methods and structs.

  Example 1 (`noun/noun`):
  Protocol method: `terminal/output`
  Trait method name: `terminal_output`
  Request/Response structs: `TerminalOutputRequest` / `TerminalOutputResponse`
  Method names struct: `terminal_output: &'static str`

  Example 2 (`noun/verb`):
  Protocol method: `terminal/new`
  Trait method name: `new_terminal`
  Request/Response structs: `NewTerminalRequest` / `NewTerminalResponse`
  Method names struct: `terminal_new: &'static str`

- Do not write any tests or docs at all!
- Add constants for the method names
- Add variants to {Agent|Client}{Request|Response} enums
- Add the methods to the Client/Agent impl of {Agent|Client}SideConnection in rust/acp.rs
- Handle the method in the decoders
- Handle the new request in the blanket impl of MessageHandler<{Agent|Client}Side>
- Add the method to markdown_generator.rs SideDocs functions
- Run `npm run generate` and fix any issues that appear
- Add the method to typescript/acp.ts classes and handlers
- Run `npm run check`
- Update the example agents and clients in tests and examples in both libraries

## Updating existing methods, their params, or output

- Update the mintlify docs and guides in the `docs` directory
- Run `npm run check` to make sure the json and zod schemas gets generated properly
- Params and responses docs make it to the schema, but the method-level docs, so make sure to update the typescript library accordingly.

Never write readme files related to the conversation unless explictly asked to.
