---
paths:
  - "rust/src/**/*.rs"
  - "rust/Cross.toml"
  - "rust/.cargo/**"
  - "rust/scripts/build*"
---

# Python to Rust CLI Porting Guide

This document covers a systematic process for porting modern Python CLI applications to

Rust for use by AI agents, maintaining feature parity through comprehensive test

fixtures.

patterns for any general and non-obvious porting situations.

## Version Requirements

- \*\*Python\*\*: 3.11+ (modern type hints for better Rust compatibility)

- \*\*Rust\*\*: 1.85+ (Edition 2024; latest stable is 1.97.1 as of August 2026; see the

  \[official stable channel\]([https://static.rust-lang.org/dist/channel-rust-stable.toml](https://static.rust-lang.org/dist/channel-rust-stable.toml))).

  Use the latest stable Rust edition for new code (2024 as of now). If your MSRV

  requires it, 2021 is acceptable.

- \*\*Testing\*\*: pytest 8.0+, cargo-tarpaulin or cargo-llvm-cov

## Version Tracking Requirements

\*\*IMPORTANT\*\*: Every Rust version must explicitly track which Python version it ports.

### Version Display Format

The Rust CLI `--version` output MUST show both versions:

\`\`\`

project-cli 0.1.0 (port of python-project 0.5.5)

\`\`\`

### Implementation

\*\*In `Cargo.toml`\*\* (workspace or CLI crate):

\`\`\`toml

\[package\]

name = "project-cli"

version = "0.1.0"

\[package.metadata.python\_source\]

# Version of Python project this Rust version is based on

# The version tag in Python already identifies the specific commit

version = "0.5.5"

\`\`\`

\*\*In CLI version output\*\* (`crates/project-cli/src/[main.rs](http://main.rs)` or similar):

\`\`\`rust

use clap::Parser;

// concat!() only accepts literals and macro invocations (like env!()),

// not const variables. env!() reads an environment variable at compile time.

// CARGO\_PKG\_VERSION is set by Cargo; PYTHON\_SOURCE\_VERSION is set by [build.rs](http://build.rs).

const VERSION\_INFO: &amp;str = concat!(

    env!("CARGO\_PKG\_VERSION"),

    " (port of python-project ",

    env!("PYTHON\_SOURCE\_VERSION"),

    ")"

);

#\[derive(Parser)\]

#\[command(version = VERSION\_INFO)\]

struct Cli {

    // ... args

}

\`\`\`

Use a [`build.rs`](http://build.rs) script to set the custom `PYTHON_SOURCE_VERSION` env var at compile

time:

\*\*[`build.rs`](http://build.rs)\*\*:

\`\`\`rust

// [build.rs](http://build.rs)

use std::process::Command;

fn main() {

    // Re-run if the Python source changes

    println!("cargo:rerun-if-changed=python-source");

    let version = Command::new("python3")

        .args(\["-c", "import myproject; print(myproject.\_\_version\_\_)"\])

        .output()

        .ok()

        .and\_then(|o| if o.status.success() {

            String::from\_utf8(o.stdout).ok()

        } else {

            None

        })

        .map(|s| s.trim().to\_string())

        .unwrap\_or\_else(|| "unknown".to\_string());

    println!("cargo:rustc-env=PYTHON\_SOURCE\_VERSION={version}");

}

\`\`\`

Alternatively, read the version from a file with a git-describe fallback:

\`\`\`rust

// [build.rs](http://build.rs)

use std::process::Command;

fn main() {

    println!("cargo:rerun-if-changed=python-source");

    let version = std::fs::read\_to\_string("python-source/VERSION")

        .map(|s| s.trim().to\_string())

        .unwrap\_or\_else(|\_| {

            // Fallback: try git describe

            Command::new("git")

                .args(\["describe", "--tags", "--always"\])

                .current\_dir("python-source")

                .output()

                .ok()

                .and\_then(|o| if o.status.success() {

                    String::from\_utf8(o.stdout).ok()

                } else {

                    None

                })

                .map(|s| s.trim().to\_string())

                .unwrap\_or\_else(|| "unknown".to\_string())

        });

    println!("cargo:rustc-env=PYTHON\_SOURCE\_VERSION={version}");

}

\`\`\`

### Documentation Requirements

1. \*\*[README.md](http://README.md)\*\* must include a “Version Correspondence” section:

   \`\`\`markdown

   ## Version Correspondence

   | Rust Version | Python Version | Date |

   |--------------|----------------|------|

   | 0.1.0        | v0.5.5         | 2024-11-02 |

   | 0.1.1        | v0.5.6         | 2024-11-15 |

   \`\`\`

2. \*\*[CHANGELOG.md](http://CHANGELOG.md)\*\* must reference Python version in each entry:

   \`\`\`markdown

   ## \[0.1.0\] - 2024-11-02

   ### Port Status

   - Ported from python-project v0.5.5

   - 100% feature parity achieved

   - All 111 tests passing

   ### Added

   - Initial Rust port

   \`\`\`

3. \*\*Git commit messages\*\* for sync updates must include:

   \`\`\`

   Sync from Python repo v0.5.6 @ def456e

   Python changes:

   - Added smart quote feature

   - Fixed list formatting edge case

   Rust updates:

   - Ported smart quote logic

   - Updated tests

   \`\`\`

### Automation in Sync Script

The sync script in \[Phase 4\](#phase-4-ongoing-synchronization-of-python-code-changes)

includes version tracking updates.

The key step is updating the `[package.metadata.python_source]` section in `Cargo.toml`.

Since `sed` cannot reliably target fields within a specific TOML section, prefer a

TOML-aware tool:

\`\`\`bash

# Using toml-cli (cargo install toml-cli):

toml set Cargo.toml package.metadata.python\_source.version "$PYTHON\_VERSION"

toml set Cargo.toml package.metadata.python\_source.commit "${LATEST\_COMMIT:0:7}"

toml set Cargo.toml package.metadata.python\_[source.date](http://source.date) "$(date +%Y-%m-%d)"

\`\`\`

If `toml-cli` is unavailable, use Python as a fallback:

\`\`\`bash

python3 -c "

import toml, sys

data = toml.load('Cargo.toml')

data\['package'\]\['metadata'\]\['python\_source'\]\['version'\] = sys.argv\[1\]

data\['package'\]\['metadata'\]\['python\_source'\]\['commit'\] = sys.argv\[2\]

with open('Cargo.toml', 'w') as f:

    toml.dump(data, f)

" "$PYTHON\_VERSION" "${LATEST\_COMMIT:0:7}"

\`\`\`

## Phase 1: Project Setup

\*\*See:\*\*

\[Initial Port Checklist - Phase 4: Project Setup\]([port-checklist-initial-template.md](http://port-checklist-initial-template.md)#phase-4-project-setup)

for complete setup instructions including repository structure, workspace configuration,

best practices configuration, and release profile optimization.

### Key Concepts

\*\*Repository Structure:\*\*

\`\`\`

project-rs/

├── python-source/              # Git submodule (read-only reference)

├── test-fixtures/              # Committed test data (self-contained)

│   ├── input/

│   └── expected/

├── crates/

│   ├── project-core/          # Library implementation

│   └── project-cli/           # Binary/CLI wrapper

├── scripts/

│   ├── [sync-from-python.sh](http://sync-from-python.sh)    # Update Python source &amp; fixtures

│   └── [validate-parity.sh](http://validate-parity.sh)     # Cross-check outputs

└── Cargo.toml                 # Workspace definition

\`\`\`

\*\*Python Source as Submodule:\*\*

Why use a submodule?

- AI agents can directly read Python source using standard file operations

- Submodule pointer records exact Python commit being ported

- Can run Python tests to verify expected behavior

- Git history shows Python version correspondence

\`\`\`bash

git submodule add [https://github.com/org/project-python.git](https://github.com/org/project-python.git) python-source

git submodule update --init --recursive

git add .gitmodules python-source/

git commit -m "Add Python source as submodule for porting reference"

\`\`\`

\*\*Important\*\*: CI/CD uses committed test fixtures, not the submodule.

## Phase 2: Dependencies &amp; Porting Process

\*\*See:\*\*

\[Initial Port Checklist - Phase 5: Dependencies &amp; Core Setup\]([port-checklist-initial-template.md](http://port-checklist-initial-template.md)#phase-5-dependencies--core-setup)

for detailed dependency setup and

\[Phase 6: Porting Process\]([port-checklist-initial-template.md](http://port-checklist-initial-template.md)#phase-6-porting-process) for

complete module-by-module porting instructions, critical pitfalls checklist, and

integration test setup.

### Dependency Mappings

| Python | Rust | Notes |

| --- | --- | --- |

| argparse/click | \[clap\]([https://docs.rs/clap](https://docs.rs/clap)) | CLI parsing |

| PyYAML | \[serde\_yaml\_ng\]([https://docs.rs/serde\_yaml\_ng](https://docs.rs/serde_yaml_ng)) | Successor to archived serde\_yaml. Also consider \[serde\_norway\]([https://docs.rs/serde\_norway](https://docs.rs/serde_norway)), which tracks a maintained libyaml fork (`unsafe-libyaml-norway`) vs serde\_yaml\_ng's unmaintained `unsafe-libyaml` |

| pytest | Built-in #\[test\] + cargo test |  |

| Markdown libs | \[comrak\]([https://github.com/kivikakk/comrak](https://github.com/kivikakk/comrak)) / \[pulldown-cmark\]([https://docs.rs/pulldown-cmark](https://docs.rs/pulldown-cmark)) | Choose based on features |

\*\*Key ecosystem crates\*\*: \[anyhow\]([https://docs.rs/anyhow](https://docs.rs/anyhow)) (app errors),

\[thiserror\]([https://docs.rs/thiserror](https://docs.rs/thiserror)) (library errors), \[serde\]([https://serde.rs/](https://serde.rs/))

(serialization)

### Porting Sequence

\*\*For each module:\*\*

1. Read Python source from `python-source/` submodule

2. Create Rust equivalent with exact function names and structurally equivalent return

   types

3. Port inline tests to Rust `#[cfg(test)]` modules

4. Validate with fixtures after each module

### Test Strategy

\*\*Integration tests\*\* in `crates/project-core/tests/`:

\`\`\`rust

#\[test\]

fn test\_format\_document\_basic() {

    let input = include\_str!("../../test-fixtures/input/[basic.md](http://basic.md)");

    let expected = include\_str!("../../test-fixtures/expected/[basic.md](http://basic.md)");

    let result = format\_document(input, &amp;Config::default());

    assert\_eq!(result, expected);

}

\`\`\`

Adjust fixture paths to your repository layout.

For large fixtures, consider reading from disk in tests to keep binaries smaller.

### Porting Parity Requirements

\*\*Important test requirements:\*\*

- Must maintain exact 1:1 replication of all Python tests (behavioral parity).

- Additional Rust-specific unit tests encouraged (faster feedback, during development of

  the port, covering Rust edge cases, confirming return types, etc.)

- Use `include_str!()` to embed fixtures (like input and expected output text files) at

  compile time

\*\*Important commenting and structure requirements:\*\*

- Must maintain clear mapping of modules in Python to modules in Rust

- Must maintain comments at the top of each module indicating which Python module it

  corresponds to

- Must maintain comments at the top of every major function indicating which Python

  function it corresponds to

### CLI Parity

\*\*See:\*\*

\[Initial Port Checklist - Phase 8: Interface Layer\]([port-checklist-initial-template.md](http://port-checklist-initial-template.md)#phase-8-interface-layer-cli-if-applicable)

for complete CLI parity requirements and validation procedures.

\*\*Key Requirements:\*\*

The Rust CLI must \*\*exactly\*\* match Python’s interface:

- Same flag names, short flags, defaults, help text

- Same exit codes (0 success, 1 errors, 2 usage)

- Accept stdin/stdout piping

- Identical error messages (or improved but consistent)

\*\*Validation:\*\*

\`\`\`bash

# Adjust the Python command to match your project (module, entry point, etc.)

uv run -q --project python-source python -m project\_cli --help &gt; python-help.txt

cargo run -- --help &gt; rust-help.txt

diff python-help.txt rust-help.txt  # Must show zero diffs

\`\`\`

\*\*Critical:\*\* CLI output must be identical byte-for-byte.

This includes help text, error messages, warnings, status messages, and all command

output (not just processing results).

## Phase 3: Testing &amp; Validation

\*\*See:\*\*

\[Initial Port Checklist - Phase 9: Testing &amp; Validation\]([port-checklist-initial-template.md](http://port-checklist-initial-template.md)#phase-9-testing--validation)

for complete testing requirements including test coverage targets, cross-validation

procedures, and quality checks.

### Cross-Validation Script

`scripts/[validate-parity.sh](http://validate-parity.sh)`:

\`\`\`bash

#!/usr/bin/env bash

set -euo pipefail

FIXTURES\_DIR="test-fixtures/input"

TEMP\_DIR="$(mktemp -d)"

cargo build --release

RUST\_BIN="target/release/project-cli"

# How to run the Python reference CLI (adjust to your project)

PY\_REF\_CMD=(uv run -q --project python-source python -m project\_cli)

pushd python-source &gt;/dev/null

uv sync -q

popd &gt;/dev/null

PASSED=0

FAILED=0

for input\_file in "$FIXTURES\_DIR"/\*.md; do

  filename="$(basename "$input\_file")"

  printf "Testing %s... " "$filename"

  "${PY\_REF\_CMD\[@\]}" "$input\_file" &gt; "$TEMP\_DIR/python-$filename"

  "$RUST\_BIN" "$input\_file" &gt; "$TEMP\_DIR/rust-$filename"

  if diff -q "$TEMP\_DIR/python-$filename" "$TEMP\_DIR/rust-$filename" &gt;/dev/null; then

    echo "PASS"

    ((PASSED++)) || true  # ((0)) returns exit code 1; guard for set -e

  else

    echo "FAIL"

    diff -u "$TEMP\_DIR/python-$filename" "$TEMP\_DIR/rust-$filename" | head -20

    ((FAILED++)) || true

  fi

done

echo "Results: $PASSED passed, $FAILED failed"

rm -rf "$TEMP\_DIR"

test $FAILED -eq 0

\`\`\`

### Coverage Targets

- ≥90% line coverage in core library

- 100% coverage of public API

- Property tests for algorithmic correctness

\*\*Tools\*\*: pytest-cov (Python); cargo-llvm-cov or cargo-tarpaulin (Rust); proptest

(property tests); criterion (benchmarks)

## Phase 4: Ongoing Synchronization of Python Code Changes

### Sync Process

`scripts/[sync-from-python.sh](http://sync-from-python.sh)`:

\`\`\`bash

#!/usr/bin/env bash

set -euo pipefail

cd python-source

git fetch origin

LATEST\_COMMIT=$(git rev-parse origin/main)

CURRENT\_COMMIT=$(git rev-parse HEAD)

if \[ "$LATEST\_COMMIT" = "$CURRENT\_COMMIT" \]; then

    echo "Already up to date with Python repo"

    exit 0

fi

echo "Updating from $CURRENT\_COMMIT to $LATEST\_COMMIT"

git checkout "$LATEST\_COMMIT"

cd ..

# Copy test fixtures

rsync -av --delete python-source/tests/fixtures/input/ test-fixtures/input/

rsync -av --delete python-source/tests/fixtures/expected/ test-fixtures/expected/

# Regenerate expected outputs with Python

# Note: --update-fixtures is project-specific; adjust to your test framework

pushd python-source &gt;/dev/null

uv sync -q

uv run -q python -m pytest tests/ --update-fixtures

popd &gt;/dev/null

cp python-source/tests/fixtures/expected/\* test-fixtures/expected/

# Update version tracking in Cargo.toml (requires toml-cli: cargo install toml-cli)

PYTHON\_VERSION=$(cd python-source &amp;&amp; git describe --tags --always)

if command -v toml &amp;&gt;/dev/null; then

    toml set Cargo.toml package.metadata.python\_source.version "$PYTHON\_VERSION" &gt; Cargo.toml.tmp \\

        &amp;&amp; mv Cargo.toml.tmp Cargo.toml

    toml set Cargo.toml package.metadata.python\_source.commit "${LATEST\_COMMIT:0:7}" &gt; Cargo.toml.tmp \\

        &amp;&amp; mv Cargo.toml.tmp Cargo.toml

    toml set Cargo.toml package.metadata.python\_[source.date](http://source.date) "$(date +%Y-%m-%d)" &gt; Cargo.toml.tmp \\

        &amp;&amp; mv Cargo.toml.tmp Cargo.toml

else

    echo "WARNING: toml-cli not found; update \[package.metadata.python\_source\] in Cargo.toml manually"

fi

CHANGE\_SUMMARY=$(cd python-source &amp;&amp; git log --oneline "$CURRENT\_COMMIT".."$LATEST\_COMMIT")

git add python-source test-fixtures/ Cargo.toml

git commit -m "Sync from Python repo @ ${LATEST\_COMMIT:0:7}

Python changes since last sync:

$CHANGE\_SUMMARY

Regenerated test fixtures and updated submodule pointer."

echo "Next steps:"

echo "1. Review Python changes: cd python-source &amp;&amp; git log $CURRENT\_COMMIT..HEAD"

echo "2. Port any new features to Rust"

echo "3. Run tests: cargo test"

echo "4. Validate parity: ./scripts/[validate-parity.sh](http://validate-parity.sh)"

\`\`\`

### Handling Python Code Updates

\*\*See:\*\* \[Subsequent Update Checklist\]([port-checklist-update-template.md](http://port-checklist-update-template.md)) for the complete

workflow when syncing Python updates to Rust.

\*\*Quick Reference - Categorize updates:\*\*

- \*\*Bug fixes\*\*: Port fix to Rust, verify tests pass

- \*\*New features\*\*: Decide if needed in Rust version

- \*\*Test additions\*\*: Port tests, implement functionality

- \*\*Refactoring\*\*: May not need Rust changes if behavior unchanged

\*\*Maintain sync log\*\* in `docs/[python-sync-log.md](http://python-sync-log.md)`:

\`\`\`markdown

## 2024-03-15: Sync from Python @ abc1234

\*\*Python changes:\*\*

- Added smart quote feature

- Fixed edge case in list formatting

\*\*Rust updates required:\*\*

- \[ \] Port smart quote logic (src/[typography.rs](http://typography.rs))

- \[ \] Port list formatting fix (src/[formatter.rs](http://formatter.rs))

- \[x\] Update tests (no code changes needed)

\*\*Status\*\*: In progress

\`\`\`

### Gradual Divergence

\*\*When to diverge:\*\*

- Rust-specific optimizations

- Performance features leveraging Rust’s strengths

- Platform-specific integrations (WebAssembly)

\*\*How to manage:\*\*

- Document differences clearly in README

- Maintain core behavioral parity

- Mark tests as “Rust-specific” or “Python-specific”

## Checklists

- \*\*\[Initial Port Checklist\]([port-checklist-initial-template.md](http://port-checklist-initial-template.md))\*\* - Complete checklist for

  porting a Python CLI to Rust for the first time

- \*\*\[Subsequent Update Checklist\]([port-checklist-update-template.md](http://port-checklist-update-template.md))\*\* - Checklist for syncing

  and porting updates from Python to existing Rust port

## Final Acceptance Criteria

\*\*Completion Gate:\*\* All items below must be satisfied.

Acceptance requires exact 100% passing of every test, exact 100% parity with every

original Python test, and exact byte-for-byte matching on all comparisons (zero diffs).

This includes test output, CLI command output, help text, error messages, and all

processing results.

\*\*See:\*\*

\[Initial Port Checklist - Final Acceptance Criteria\]([port-checklist-initial-template.md](http://port-checklist-initial-template.md)#final-acceptance-criteria)

for the complete, detailed checklist.

### Summary

The acceptance criteria cover:

- \*\*Porting Parity Requirements (Mandatory):\*\* All Python tests replicated with exact

  1:1 behavioral parity, module/function mapping comments, proper use of

  `include_str!()` for fixtures

- \*\*CLI Parity (Mandatory for CLI apps):\*\* Exact flag names/defaults/help text, zero

  diffs on all CLI output (help, errors, warnings, status), correct exit codes,

  stdin/stdout piping

- \*\*Test Parity (Mandatory):\*\* 100% of tests pass, every original Python test replicated

  with exact behavior, all output comparisons are byte-for-byte matches (zero diffs)

- \*\*Cross-Validation (Mandatory):\*\* Zero diffs across all fixtures, byte-for-byte

  matching of all file outputs

- \*\*Performance Targets:\*\* Binary size &lt; 10MB, processing speed \~20-40x faster than

  Python (measured in the flowmark port; 10-50x typical), startup time &lt; 50ms

- \*\*Quality Metrics (Mandatory):\*\* Zero clippy warnings, ≥90% test coverage in core

  library, 100% coverage of public API, all public APIs documented, security audits pass

## Key Non-Obvious Pitfalls

### Regex Pattern Differences

\*\*Problem:\*\* Python’s `re.match()` implicitly anchors to start; Rust’s `is_match()`

matches anywhere.

\`\`\`python

# Python - matches only at start

pattern = r"\\w\*\[sS\]'$"

re.match(pattern, "James'")      # Matches

re.match(pattern, "$James'")     # Does NOT match

\`\`\`

\`\`\`rust

// Rust - WITHOUT explicit anchor

let pattern = Regex::new(r"\\w\*\[sS\]'$").unwrap();

[pattern.is](http://pattern.is)\_match("$James'")      // ALSO matches (incorrect!)

// Rust - WITH explicit anchor (correct)

let pattern = Regex::new(r"^\\w\*\[sS\]'$").unwrap();

[pattern.is](http://pattern.is)\_match("$James'")      // Does NOT match

\`\`\`

\*\*Takeaway:\*\* Any pattern used with `re.match()` needs `^` anchor in Rust.

### Regex Feature Parity (look-arounds, backrefs)

Python’s `re` supports look-around and backreferences; the Rust `regex` crate does not.

If parity is required, use `fancy-regex` for those patterns, understanding it can be

slower; keep hot paths on `regex` where possible.

### String Preservation

\*\*Problem:\*\* Don’t “helpfully” transform input when Python doesn’t.

\`\`\`python

def first\_sentence(text: str) -&gt; str:

    sentences = split\_sentences(text)

    return sentences\[0\] if sentences else text  # Original text unchanged

\`\`\`

\`\`\`rust

// WRONG - modifies input

pub fn first\_sentence(text: &amp;str) -&gt; String {

    if [sentences.is](http://sentences.is)\_empty() {

        text.trim().to\_string()  // WRONG!

    } else { sentences\[0\].clone() }

}

// CORRECT - preserves input

pub fn first\_sentence(text: &amp;str) -&gt; String {

    if [sentences.is](http://sentences.is)\_empty() {

        [text.to](http://text.to)\_string()  // Unchanged

    } else { sentences\[0\].clone() }

}

\`\`\`

\*\*Test case:\*\* `assert first_sentence(" ") == " " # Not ""!`

### Line Endings and Encodings

- Normalize CRLF/LF in tests when line endings are not semantically significant

- Assume UTF-8; reject or normalize invalid encodings at boundaries

- For byte-for-byte parity tests, compare exact bytes and document expectations

### Arena Pattern for AST Nodes

\*\*Problem:\*\* Tree structures create self-referential lifetimes.

\`\`\`rust

// DOESN'T work - self-referential struct

pub struct MarkdownDocument&lt;'a&gt; {

    arena: Arena&lt;AstNode&lt;'a&gt;&gt;,     // Owns the arena

    root: &amp;'a AstNode&lt;'a&gt;,         // Borrows from arena - IMPOSSIBLE

}

\`\`\`

\*\*Solution:\*\* Closure-based API with higher-rank trait bounds:

\`\`\`rust

pub fn with\_markdown\_ast&lt;F, R&gt;(text: &amp;str, f: F) -&gt; Result&lt;R&gt;

where

    F: for&lt;'a&gt; FnOnce(&amp;'a AstNode&lt;'a&gt;) -&gt; Result&lt;R&gt;,

{

    let arena = Arena::new();

    let options = Options::default();

    let root = parse\_document(&amp;arena, text, &amp;options);

    f(root)

}  // Arena drops after user code completes

\`\`\`

\*\*Takeaway:\*\* Use closures to control borrow scope; common with comrak, pulldown-cmark.

### Function Signature Compatibility

\*\*Problem:\*\* Even small differences in return types break compatibility.

\`\`\`python

def split\_frontmatter(text: str) -&gt; tuple\[str, str\]:

    return ("", text)  # Returns ("", text) if no frontmatter

\`\`\`

\`\`\`rust

// WRONG - different return type

pub fn extract\_frontmatter(text: &amp;str) -&gt; Result&lt;Option&lt;(String, String)&gt;&gt; {

    Ok(None)  // NOT COMPATIBLE

}

// CORRECT - matches Python

pub fn split\_frontmatter(text: &amp;str) -&gt; (String, String) {

    ("".to\_string(), [text.to](http://text.to)\_string())

}

\`\`\`

\*\*Takeaway:\*\* Function names, return types, and empty/None cases must match exactly.

### Unicode Handling

\*\*Note:\*\* Rust supports Unicode string literals.

Using explicit escapes in tests avoids encoding/visual issues and makes intent

unambiguous.

\`\`\`python

assert smart\_quotes("I'm there") == "I’m there"  # Direct Unicode literal is fine

\`\`\`

\`\`\`rust

assert\_eq!(smart\_quotes("I'm there"), "I\\u{2019}m there");  // \\u{2019} = '

\`\`\`

\*\*Common escapes:\*\* `\u{2018}` ‘, `\u{2019}` ’, `\u{201c}` “, `\u{201d}` ”, `\u{2026}` …

### Test Output Validation

\*\*Problem:\*\* Internal state changes don’t guarantee correct output.

\`\`\`rust

// WRONG - counting nodes

let strong\_before = count\_strong\_nodes(root);

unbold\_headings(root);

assert\_eq!(count\_strong\_nodes(root), strong\_before - 4);  // Could still render wrong!

// CORRECT - compare full rendered output byte-for-byte

let arena = Arena::new();

let root = parse\_document(&amp;arena, INPUT, &amp;options);

unbold\_headings(root);

let mut output = Vec::new();

format\_commonmark(root, &amp;options, &amp;mut output).unwrap();

let result = String::from\_utf8(output).unwrap();

assert\_eq!(result.trim(), EXPECTED\_OUTPUT.trim());  // Exact match

\`\`\`

\*\*Takeaway:\*\* Prefer exact byte-for-byte comparisons.

Only trim if trailing whitespace is explicitly non-semantic for your project.

### Integration Test Location

\*\*Problem:\*\* Cargo only discovers integration tests inside a crate’s `tests/` directory.

A `tests/` directory at the workspace root is ignored unless the root `Cargo.toml` is

also a `[package]` (not just a `[workspace]`).

\`\`\`

# WRONG - workspace-only root; tests/ NOT discovered by cargo test

project-rs/

├── Cargo.toml              # \[workspace\] only, no \[package\]

├── tests/

│   └── test\_[cleanups.rs](http://cleanups.rs)

└── crates/project-core/src/

# CORRECT - crate's tests/ discovered

project-rs/

└── crates/project-core/

    ├── tests/test\_[cleanups.rs](http://cleanups.rs)

    └── src/

\`\`\`

\*\*Takeaway:\*\* Integration tests go in `<crate>/tests/`, not workspace root.

### Error Reporting Consistency

- Use `thiserror` for library error types and `color-eyre` (or similar) in the CLI for

  consistent, readable diagnostics

- Send human-readable diagnostics to stderr; reserve stdout for program output

- Match Python’s exit codes and error phrasing where parity is required

### Error Handling for Compatibility

\`\`\`rust

// When Python never raises, don't use Result

pub fn split\_frontmatter(text: &amp;str) -&gt; (String, String) {

    // No Result needed - matches Python's behavior

    ("".to\_string(), [text.to](http://text.to)\_string())

}

\`\`\`

\*\*Takeaway:\*\* Only use `Result` if Python function can raise.

### Other Common Issues

- \*\*String slicing\*\*: `text[start..end]` panics if indices are not on UTF-8 char

  boundaries; use `.char_indices()` to find safe boundaries, or `.chars()` to iterate by

  character

- \*\*Library types\*\*: `comrak` inner types change between versions (e.g.,

  `NodeValue::Text` was `Vec<u8>` pre-0.4, `String` in 0.4-0.44, and

  `Cow<'static, str>` in 0.45+). Check docs for your pinned version and convert

  explicitly.

- \*\*Module system\*\*: Rust requires explicit `mod` declarations

- \*\*Unit test timing\*\*: Port function → write unit tests → run/fix → move to next →

  integration tests last

## Useful Patterns

### Closure-based API for Borrowed Data

\`\`\`rust

pub fn with\_resource&lt;F, R&gt;(f: F) -&gt; Result&lt;R&gt;

where

    F: for&lt;'a&gt; FnOnce(&amp;'a Resource&lt;'a&gt;) -&gt; Result&lt;R&gt;,

{

    let resource = create\_resource();

    f(&amp;resource)

}

\`\`\`

\*\*Use when:\*\* Resource has internal borrows (like Arena + AST nodes)

### Lazily Compiled Regex

\`\`\`rust

use std::sync::LazyLock;

use regex::Regex;

static PATTERN: LazyLock&lt;Regex&gt; =

    LazyLock::new(|| Regex::new(r"pattern").unwrap());

pub fn process(text: &amp;str) -&gt; String {

    PATTERN.replace\_all(text, "replacement").to\_string()

}

\`\`\`

\*\*Use when:\*\* Regex pattern used multiple times (compilation is expensive)

### Build size and performance tips

- In `Cargo.toml` release profile: enable LTO, reduce codegen units, consider

  `panic = "abort"`

- For smaller binaries: use `opt-level = "z"` or `"s"`, and strip symbols in CI

- Avoid heavy dependencies in the CLI crate; keep them in the library when reusable

- Benchmark with `criterion` and inspect size with `cargo-bloat`