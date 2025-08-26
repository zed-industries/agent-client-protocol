All paths in the protocol should be absolute

When updating JSON-RPC methods, their params, or output:

- Update the mintlify docs and guides in the `docs` directory
- Run `npm run check` to make sure the json and zod schemas gets generated properly
- Params and responses docs make it to the schema, but the method-level docs, so make sure to update the typescript library accordingly.
