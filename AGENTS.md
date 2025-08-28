All paths in the protocol should be absolute

When I ask you to add a new ACP Agent or Client method:

- Create empty params and output structs in rust/client.rs or rust/agent.rs under the corresponding section. I'll add the fields myself.
- Do not write any tests or docs at all!
- Add constants for the method names
- Add variants to {Agent|Client}{Request|Response} enums
- Add the methods to the Client/Agent impl of {Agent|Client}SideConnection in rust/acp.rs
- Handle the new request in the blanket impl of MessageHandler<{Agent|Client}Side>
- Add the method to markdown_generator.rs SideDocs functions
- Run `npm run generate` and fix any issues that appear
- Add the method to typescript/acp.ts classes and handlers
- Run `npm run check`

When updating existing JSON-RPC methods, their params, or output:

- Update the mintlify docs and guides in the `docs` directory
- Run `npm run check` to make sure the json and zod schemas gets generated properly
- Params and responses docs make it to the schema, but the method-level docs, so make sure to update the typescript library accordingly.

Never write readme files related to the conversation unless explictly asked to.
