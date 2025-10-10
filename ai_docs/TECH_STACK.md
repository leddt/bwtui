# Technology Stack

## Language: Rust

### Why Rust?

1. **Performance** - Compiled, zero-cost abstractions, no garbage collection
2. **Memory Safety** - No data races, no null pointers, no buffer overflows
3. **Concurrency** - Fearless concurrency with ownership system
4. **Ecosystem** - Excellent TUI libraries and CLI tooling
5. **Binary Distribution** - Single statically-linked binary, easy to distribute

### Alternatives Considered

| Language | Pros | Cons | Verdict |
|----------|------|------|---------|
| **Python** | Fast development, textual library | Slower, requires Python runtime | Not chosen - performance concerns |
| **Go** | Simple, fast compilation, bubbletea | Less mature TUI ecosystem | Good alternative |
| **C++** | Maximum performance | Complex, harder to maintain | Overkill |

## Core Dependencies

### TUI Framework: ratatui (v0.24+)

**Why ratatui?**
- Modern fork of tui-rs with active maintenance
- Immediate mode rendering (no state management in widgets)
- Excellent documentation and examples
- Rich widget library (lists, tables, charts, etc.)
- Flexible layout system
- Unicode and wide character support

**Alternatives:**
- `cursive` - More widget-focused, batteries included, but less flexible
- `termion` - Lower-level, more control, but more work
- `crossterm` - Just terminal manipulation, not a full TUI framework

### Terminal Backend: crossterm (v0.27+)

**Why crossterm?**
- Cross-platform (Windows, Linux, macOS)
- Works with ratatui
- Low-level terminal control
- Event handling (keyboard, mouse, resize)
- Raw mode and alternate screen support

### Async Runtime: tokio (v1.35+)

**Why tokio?**
- Industry standard for async Rust
- Non-blocking I/O for CLI operations
- Excellent performance
- Rich ecosystem
- Built-in utilities (timers, channels, etc.)

**Use cases:**
- Executing bw CLI commands without blocking UI
- Clipboard auto-clear timers
- Background vault syncing
- TOTP refresh

### CLI Integration: tokio-process

**Why tokio-process?**
- Async process spawning
- Works seamlessly with tokio
- Non-blocking I/O for stdout/stderr
- Easy integration with Bitwarden CLI

**Alternative approach:**
- Direct API calls to Bitwarden server (not chosen - more complex, requires auth handling)

### Clipboard: arboard (v3.3+)

**Why arboard?**
- Cross-platform (Windows, Linux, macOS)
- Simple API
- Reliable
- No external dependencies on most platforms

**Alternatives:**
- `clipboard` - Less actively maintained
- `copypasta` - More complex API

### Filtering: fuzzy-matcher (v0.3+)

**Why fuzzy-matcher?**
- Fast fuzzy matching algorithm
- Multiple algorithms (skim, clangd)
- Good for interactive search
- Low overhead

**Use case:**
- Filter vault entries as user types
- Match partial queries against name/domain/username

### Parallel Processing: rayon (v1.8+)

**Why rayon?**
- Data parallelism made easy
- Work-stealing scheduler
- Zero-cost abstraction

**Use case:**
- Parallel filtering for large vaults (>1000 items)
- Only used when vault is large enough to benefit

### Configuration: config + directories

**Why these libraries?**
- `config` - Multiple format support (TOML, JSON, YAML)
- `directories` - Cross-platform config directory location
- Standard approach for Rust CLI apps

### Error Handling: anyhow + thiserror

**Why both?**
- `thiserror` - Derive macros for error types (library code)
- `anyhow` - Ergonomic error handling (application code)
- Both are industry standard

### Logging: tracing + tracing-subscriber

**Why tracing?**
- More powerful than `log` crate
- Structured logging
- Async-aware
- Good for debugging

### Serialization: serde + serde_json

**Why serde?**
- Industry standard
- Zero-cost abstractions
- Derive macros for easy serialization
- Excellent JSON support for bw CLI output

## Architecture Decisions

### 1. Bitwarden CLI vs Direct API

**Decision**: Use Bitwarden CLI (`bw`)

**Rationale:**
- Official, maintained by Bitwarden
- Handles authentication and session management
- No need to implement crypto ourselves
- Users already have it installed
- Simpler and more secure

**Trade-offs:**
- Requires spawning processes
- Slightly higher latency
- Dependent on CLI being installed

### 2. Async vs Sync

**Decision**: Async with tokio

**Rationale:**
- Non-blocking UI during CLI operations
- Better performance for concurrent operations
- Modern Rust best practice

**Trade-offs:**
- Slightly more complex code
- Async ecosystem overhead

### 3. Immediate Mode vs Retained Mode UI

**Decision**: Immediate mode (ratatui)

**Rationale:**
- Simpler mental model
- Full control over rendering
- Less state management
- Better for dynamic UIs

**Trade-offs:**
- Re-renders entire screen each frame
- More manual layout management

### 4. Filtering Strategy

**Decision**: Fuzzy matching with fallback to exact match

**Rationale:**
- Better UX for quick searches
- Configurable for user preference
- Fast enough for large vaults

**Trade-offs:**
- More complex implementation
- Slightly slower than exact match

### 5. Data Caching

**Decision**: In-memory cache with manual refresh

**Rationale:**
- Faster than fetching every time
- Vault rarely changes during session
- User controls when to sync

**Trade-offs:**
- Stale data if changed elsewhere
- Memory usage for large vaults

## Performance Characteristics

### Startup Time
- **Target**: <200ms
- **Factors**: CLI check, vault load, UI init
- **Optimization**: Parallel initialization, lazy loading

### Filtering Performance
- **Target**: <16ms (60fps)
- **Factors**: Vault size, algorithm complexity
- **Optimization**: Incremental filtering, parallel processing for large vaults

### Memory Usage
- **Target**: <50MB for typical vault (100 entries)
- **Factors**: Vault size, cached data
- **Optimization**: Efficient data structures, lazy loading

### Binary Size
- **Target**: <5MB (release build with optimizations)
- **Factors**: Dependencies, features
- **Optimization**: `strip`, `lto`, minimal dependencies

## Build Configuration

### Cargo.toml Optimizations

```toml
[profile.release]
opt-level = 3          # Maximum optimization
lto = "fat"            # Link-time optimization
codegen-units = 1      # Better optimization (slower compile)
strip = true           # Strip symbols
panic = "abort"        # Smaller binary
```

### Feature Flags

```toml
[features]
default = ["fuzzy"]
fuzzy = ["fuzzy-matcher"]       # Fuzzy matching support
clipboard = ["arboard"]         # Clipboard integration
totp = []                       # TOTP support (via CLI)
```

## Cross-Platform Considerations

### Windows
- Uses Windows API for clipboard
- PowerShell for bw CLI execution
- ANSI escape sequence support (Windows 10+)

### Linux
- X11/Wayland clipboard support
- Standard shell execution
- Wide terminal emulator support

### macOS
- Pasteboard for clipboard
- zsh/bash for execution
- Terminal.app and iTerm2 support

## Security Considerations

### Session Management
- Never store session tokens on disk
- Use environment variables or memory only
- Clear tokens on exit

### Clipboard Security
- Auto-clear after timeout
- Option to disable clipboard history
- No clipboard logging

### Data Handling
- No local vault storage
- All data in memory only
- Secure process execution

### Dependencies
- Regular security audits (`cargo audit`)
- Minimal dependencies
- Well-maintained libraries only

## Development Tools

### Required
- Rust 1.70+ (for MSRV)
- Cargo
- Bitwarden CLI

### Recommended
- `cargo-watch` - Auto-rebuild on changes
- `cargo-audit` - Security vulnerability scanning
- `cargo-clippy` - Linting
- `cargo-fmt` - Code formatting
- `bacon` - Background continuous testing

### CI/CD
- GitHub Actions
- Cross-platform testing
- Automated releases
- Security scanning

## Deployment

### Distribution Methods

1. **Cargo Install**
   ```bash
   cargo install bwtui
   ```

2. **Binary Releases**
   - GitHub Releases
   - Platform-specific installers

3. **Package Managers**
   - Homebrew (macOS/Linux)
   - AUR (Arch Linux)
   - Scoop (Windows)
   - APT/DNF (Debian/Fedora)

## Conclusion

This tech stack provides:
- ✅ High performance
- ✅ Cross-platform support
- ✅ Memory safety
- ✅ Modern development experience
- ✅ Easy distribution
- ✅ Secure by default
- ✅ Maintainable codebase

The combination of Rust, ratatui, and tokio creates a solid foundation for a fast, reliable, and user-friendly Bitwarden TUI application.

