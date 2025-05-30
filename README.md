# RegexGen - Topic Annotation Tool

A WebAssembly-powered tool for creating regex patterns from text selections. Built with Rust and designed with Verint-inspired styling.

## Features

- **Text Selection**: Click or double-click to select whole words from input text
- **Regex Generation**: Automatically generates regex patterns with word boundaries
- **Topic Management**: Save named topics (regex patterns) to browser local storage
- **Topic Testing**: Test saved regex patterns against new text
- **Persistent Storage**: All topics are saved in browser local storage

## Prerequisites

- Rust (latest stable version)
- wasm-pack (will be installed automatically by build script)

## Building

1. Run the build script:
   ```bash
   ./build.sh
   ```

   This will:
   - Install wasm-pack if needed
   - Build the WebAssembly module
   - Create a Python development server script

## Running

1. Start the development server:
   ```bash
   ./serve.py
   ```

2. Open your browser to `http://localhost:8000`

## Usage

1. **Creating Patterns**:
   - Paste or type text in the main text area
   - Click words to select them:
     - Adjacent selections become phrases (matched as exact strings)
     - Non-adjacent selections create AND patterns (both must exist)
   - Enter a pattern name
   - Click "Save Pattern" to create and save

2. **Testing Patterns**:
   - Click "Test" button next to any saved pattern
   - Results appear in the Test Results panel
   - Matches are highlighted in yellow

3. **Managing Patterns**:
   - View all saved patterns in the sidebar
   - Each pattern shows its structure visually
   - Delete patterns using the Delete button

## Future Enhancements

- Topics: Collections of patterns grouped together
- Pattern composition: Creating patterns from other patterns
- NOT logic for exclusion patterns
- Text history management
- Export/import functionality
- OR patterns (alternative word matching)

## Technical Details

- Built with Rust and WebAssembly
- Uses `wasm-bindgen` for JavaScript interop
- Regex patterns include word boundaries (`\b`)
- Topics persist in browser local storage