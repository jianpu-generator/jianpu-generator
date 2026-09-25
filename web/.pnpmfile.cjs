// openapi-typescript (the Synced Share worker client's generator, see the
// `build:worker-types` script) drives the TypeScript 5 JS compiler API,
// which TypeScript 7 -- this project's own `typescript` -- no longer ships.
// Swap its `typescript` peer (which would resolve to our TS 7) for its own
// pinned TS 5 dependency.
module.exports = {
  hooks: {
    readPackage(pkg) {
      if (pkg.name === 'openapi-typescript') {
        delete pkg.peerDependencies?.typescript
        pkg.dependencies = { ...pkg.dependencies, typescript: '5.9.3' }
      }
      return pkg
    },
  },
}
