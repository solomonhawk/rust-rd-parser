# TBL Language Support

This VS Code extension provides syntax highlighting for TBL (Table) files used in random generation systems.

## Features

- **Syntax Highlighting**: Full syntax highlighting for TBL language constructs
- **Comments**: Support for both line (`//`) and block (`/* */`) comments
- **Table Declarations**: Highlighting for table identifiers and flags
- **Dice Expressions**: Special highlighting for dice roll expressions (`d6`, `2d10`, etc.)
- **Table References**: Highlighting for table reference expressions (`{#table}`)
- **Auto-completion**: Basic bracket and quote auto-completion

## TBL Language Support

The TBL language supports:

- **Table Declarations**: `#table_name[flags]`
- **Rules**: `weight: rule text with {expressions}`
- **Dice Rolls**: `{d6}`, `{2d10}`, `{100d20}`
- **Table References**: `{#other_table}`
- **Table Reference Modifiers**: `{#table|indefinite}`, `{#table|capitalize}`, `{#table|indefinite|capitalize}`
- **Comments**: `// line comment` and `/* block comment */`
- **Export Flags**: `#table[export]`

### Table Reference Modifiers

The TBL language supports modifiers that transform table reference output:

```tbl
#animal
1.0: cat
1.0: dog
1.0: bird

#description
1.0: You see {#animal|indefinite}
2.0: {#animal|definite|capitalize} approaches
3.0: A wild {#animal|uppercase} appears!
```

**Available Modifiers:**
- `indefinite` - Adds "a" or "an" prefix
- `definite` - Adds "the" prefix
- `capitalize` - Capitalizes first letter
- `uppercase` - Converts to uppercase
- `lowercase` - Converts to lowercase

## Example

```tbl
// Weapon generation table
#weapons[export]
1.0: Sword (damage: {2d6} + {1d4})
2.0: Bow (damage: {1d8})
1.5: Dagger (damage: {1d4})

/*
   Material table for armor generation
*/
#materials
1.0: leather
2.0: iron
1.5: steel

// Armor references materials
#armor
1.0: {#materials} armor
2.0: {#materials} shield
```

## Installation

This extension can be installed locally for development or packaged for distribution.

## Development

Built for the rust-rd-parser project to provide IDE support for TBL files.
