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

1. **Creating Topics**:
   - Paste or type text in the main text area
   - Select words by clicking/double-clicking or highlighting
   - Enter a topic name
   - Click "Generate Regex" to create and save the topic

2. **Testing Topics**:
   - Select a saved topic from the dropdown
   - Enter test text in the main area
   - Click "Test Topic" to see matches

3. **Managing Topics**:
   - View all saved topics in the sidebar
   - Test or delete topics using the action buttons

## Future Enhancements

- Support for phrase selection (multiple words)
- AND/AND NOT logic for combining patterns
- Text history management
- Export/import functionality

## Technical Details

- Built with Rust and WebAssembly
- Uses `wasm-bindgen` for JavaScript interop
- Regex patterns include word boundaries (`\b`)
- Topics persist in browser local storage