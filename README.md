# RustDBGrid

Cross-platform database manager built with Rust, Tauri, and Svelte.

## Features

- **Multi-database support**: MySQL, PostgreSQL, MongoDB, Redis, Apache Ignite
- **SQL Editor**: Execute queries with syntax highlighting
- **Data Grid**: View query results in a table format
- **Schema Explorer**: Browse databases, tables, and columns
- **Export**: Export schema and data (coming soon)

## Tech Stack

- **Backend**: Rust with Tauri
- **Frontend**: Svelte with Bootstrap and Font Awesome
- **Database Drivers**: sqlx, mongodb, redis

## Prerequisites

- Rust (latest stable)
- Node.js 18+ and npm
- Platform-specific dependencies for Tauri

## Getting Started

### Install Dependencies

```bash
npm install
```

### Development

Run the application in development mode:

```bash
npm run tauri dev
```

### Build

Build the application for production:

```bash
npm run tauri build
```

## Project Structure

```
rustdbgrid/
├── src/                    # Svelte frontend
│   ├── components/         # UI components
│   ├── stores/             # State management
│   ├── utils/              # Utility functions
│   └── App.svelte          # Main app component
├── src-tauri/              # Rust backend
│   ├── src/
│   │   ├── commands/       # Tauri commands
│   │   ├── db/             # Database drivers
│   │   ├── models/         # Data structures
│   │   └── main.rs         # Entry point
│   └── Cargo.toml          # Rust dependencies
└── package.json            # Node dependencies
```

## Current Status

✅ MySQL support (basic)
⏳ PostgreSQL (not yet implemented)
⏳ MongoDB (not yet implemented)
⏳ Redis (not yet implemented)
⏳ Apache Ignite (not yet implemented)
✅ Basic SQL editor
✅ Data grid display
✅ Connection management
⏳ Schema export
⏳ Data export
⏳ Copy to clipboard

## License

MIT
