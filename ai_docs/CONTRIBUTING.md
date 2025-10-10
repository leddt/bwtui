# Contributing to bwtui

Thank you for your interest in contributing to bwtui! This document provides guidelines and instructions for contributing.

## Code of Conduct

Be respectful, inclusive, and professional. We're all here to build something great together.

## Getting Started

### Prerequisites

- Rust 1.70 or later
- Bitwarden CLI (`bw`) installed
- Git
- A terminal emulator

### Development Setup

1. **Clone the repository**
   ```bash
   git clone https://github.com/yourusername/bwtui.git
   cd bwtui
   ```

2. **Build the project**
   ```bash
   cargo build
   ```

3. **Run tests**
   ```bash
   cargo test
   ```

4. **Run the application**
   ```bash
   cargo run
   ```

### Recommended Tools

- `cargo-watch` - Auto-rebuild on changes
  ```bash
  cargo install cargo-watch
  cargo watch -x run
  ```

- `cargo-clippy` - Linting
  ```bash
  cargo clippy
  ```

- `cargo-fmt` - Code formatting
  ```bash
  cargo fmt
  ```

- `bacon` - Background continuous testing
  ```bash
  cargo install bacon
  bacon
  ```

## Development Workflow

### 1. Create an Issue

Before starting work, create or find an issue describing what you want to work on. This helps avoid duplicate work and allows for discussion.

### 2. Fork and Branch

1. Fork the repository
2. Create a feature branch:
   ```bash
   git checkout -b feature/your-feature-name
   # or
   git checkout -b fix/your-bug-fix
   ```

### 3. Make Your Changes

- Write clean, readable code
- Follow Rust conventions and idioms
- Add tests for new functionality
- Update documentation as needed
- Keep commits focused and atomic

### 4. Test Your Changes

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_name

# Run clippy
cargo clippy -- -D warnings

# Format code
cargo fmt

# Check formatting
cargo fmt -- --check
```

### 5. Commit Your Changes

Write clear, descriptive commit messages:

```
feat: add TOTP auto-refresh feature

- Implement auto-refresh timer for TOTP codes
- Add configuration option for refresh interval
- Update UI to show refresh countdown
```

Use conventional commit prefixes:
- `feat:` - New feature
- `fix:` - Bug fix
- `docs:` - Documentation changes
- `style:` - Code style changes (formatting, etc.)
- `refactor:` - Code refactoring
- `perf:` - Performance improvements
- `test:` - Adding or updating tests
- `chore:` - Maintenance tasks

### 6. Push and Create Pull Request

```bash
git push origin feature/your-feature-name
```

Then create a Pull Request on GitHub with:
- Clear title and description
- Link to related issue(s)
- Screenshots/recordings (for UI changes)
- Testing steps

## Code Style Guidelines

### Rust Conventions

1. **Follow standard Rust style**
   - Use `cargo fmt` (rustfmt)
   - Follow the [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)

2. **Naming conventions**
   ```rust
   // Types: PascalCase
   struct VaultItem { }
   enum ItemType { }
   
   // Functions and variables: snake_case
   fn list_items() { }
   let selected_index = 0;
   
   // Constants: SCREAMING_SNAKE_CASE
   const MAX_ENTRIES: usize = 1000;
   ```

3. **Error handling**
   ```rust
   // Use Result<T> for fallible operations
   fn load_config() -> Result<Config> { }
   
   // Use proper error types
   Err(BwError::VaultLocked)
   ```

4. **Documentation**
   ```rust
   /// Fetches all vault items from Bitwarden CLI.
   ///
   /// # Errors
   ///
   /// Returns an error if the vault is locked or CLI command fails.
   pub async fn list_items(&self) -> Result<Vec<VaultItem>> {
       // ...
   }
   ```

### Code Organization

1. **Module structure**
   - One main concept per module
   - Public API at module root
   - Implementation details in submodules

2. **Import organization**
   ```rust
   // Standard library
   use std::io;
   
   // External crates
   use tokio::process::Command;
   use ratatui::widgets::List;
   
   // Internal modules
   use crate::error::Result;
   use crate::types::VaultItem;
   ```

3. **File size**
   - Keep files under 500 lines when possible
   - Split large modules into submodules

### UI Guidelines

1. **Responsive design**
   - Handle small terminal sizes (80x24 minimum)
   - Gracefully handle window resizing

2. **Keyboard shortcuts**
   - Follow common conventions (vim-style for navigation)
   - Document all shortcuts
   - Make shortcuts configurable when possible

3. **Error messages**
   - User-friendly, actionable messages
   - Include suggested fixes when possible
   - Don't expose internal details to users

4. **Performance**
   - Keep UI responsive (<16ms per frame)
   - Use async for long operations
   - Show loading indicators

## Testing Guidelines

### Unit Tests

Write unit tests for:
- Pure functions
- Business logic
- Error handling
- Edge cases

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter_by_name() {
        let items = vec![/* ... */];
        let filter = Filter::new(false, false);
        let result = filter.apply(&items, "example");
        assert_eq!(result.len(), 1);
    }
}
```

### Integration Tests

Place integration tests in `tests/` directory:

```rust
// tests/cli_integration.rs
use bwtui::cli::BitwardenCli;

#[tokio::test]
async fn test_list_items_when_unlocked() {
    // Test implementation
}
```

### Manual Testing Checklist

Before submitting a PR, test:
- [ ] Functionality works as expected
- [ ] UI renders correctly on different terminal sizes
- [ ] Keyboard shortcuts work
- [ ] Error handling works
- [ ] No panics or crashes
- [ ] Performance is acceptable

## Documentation Guidelines

### Code Documentation

1. **Public APIs must have doc comments**
   ```rust
   /// Copies the provided text to the system clipboard.
   ///
   /// The clipboard will be automatically cleared after the configured timeout.
   ///
   /// # Arguments
   ///
   /// * `text` - The text to copy to the clipboard
   ///
   /// # Errors
   ///
   /// Returns an error if clipboard access fails.
   pub async fn copy_with_timeout(&mut self, text: String) -> Result<()> {
       // ...
   }
   ```

2. **Complex algorithms should have explanatory comments**
   ```rust
   // Use fuzzy matching with SkimMatcherV2 for better results
   // with partial queries. Fall back to exact match if fuzzy
   // matching is disabled in config.
   ```

### User Documentation

Update relevant documentation when adding features:
- README.md - Quick start and overview
- ARCHITECTURE.md - Technical details
- IMPLEMENTATION.md - Implementation plan
- User guides - How-to documentation

## Performance Guidelines

### Benchmarking

Before and after performance-related changes:

```bash
# Add benchmarks in benches/
cargo bench
```

### Profiling

For CPU profiling:
```bash
cargo build --release
# Use your preferred profiler (perf, instruments, etc.)
```

### Performance Targets

- Startup: <200ms
- Filtering: <16ms (for 60fps)
- Memory: <50MB for typical vault (100 entries)

## Pull Request Process

### Before Submitting

1. ✅ Tests pass: `cargo test`
2. ✅ No clippy warnings: `cargo clippy -- -D warnings`
3. ✅ Code is formatted: `cargo fmt`
4. ✅ Documentation is updated
5. ✅ Commits are clean and descriptive
6. ✅ Manual testing completed

### PR Description Template

```markdown
## Description
Brief description of changes

## Related Issue
Fixes #123

## Changes
- Change 1
- Change 2

## Testing
How to test these changes

## Screenshots (if applicable)
[Add screenshots or recordings]

## Checklist
- [ ] Tests added/updated
- [ ] Documentation updated
- [ ] Code formatted
- [ ] No clippy warnings
- [ ] Manual testing completed
```

### Review Process

1. Maintainer reviews the PR
2. Address any requested changes
3. Once approved, maintainer will merge

## Areas Where We Need Help

### High Priority
- Cross-platform testing (especially Windows)
- Performance optimization
- Bug fixes
- Documentation improvements

### Medium Priority
- New features (see ROADMAP.md)
- UI/UX improvements
- Test coverage
- Examples and tutorials

### Low Priority
- Code refactoring
- Additional themes
- Extra features

## Getting Help

### Questions?
- Open a [Discussion](https://github.com/yourusername/bwtui/discussions)
- Join our community chat (if available)

### Found a Bug?
- Check [existing issues](https://github.com/yourusername/bwtui/issues)
- If not found, create a new issue with:
  - Steps to reproduce
  - Expected behavior
  - Actual behavior
  - System information (OS, Rust version, etc.)

### Feature Request?
- Open a [Feature Request issue](https://github.com/yourusername/bwtui/issues/new)
- Describe the feature and use case
- Discuss with maintainers before implementing

## Release Process (for maintainers)

1. Update version in `Cargo.toml`
2. Update CHANGELOG.md
3. Create git tag: `git tag v1.0.0`
4. Push tag: `git push origin v1.0.0`
5. GitHub Actions will build and create release
6. Announce release

## License

By contributing, you agree that your contributions will be licensed under the MIT License.

## Recognition

Contributors will be recognized in:
- README.md contributors section
- Release notes
- Project website (if applicable)

Thank you for contributing to bwtui! 🎉

