# License text supplements

Some published dependencies omit license files. `overrides.json` maps exact package versions to upstream license text from their recorded source commits, or to the same-version parent npm package that distributes the platform binary. Source URLs are retained in the vendored files.

The selectors crate declares MPL-2.0. Its unchanged corresponding source archive is linked explicitly in the generated notices and dependency inventory, along with the Mozilla license text. Updating a dependency requires reviewing its new license and any versioned override; the notice generator fails if a dependency has no available license text.
