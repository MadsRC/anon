# Test Driven Development Instructions

## MANDATORY Red-Green-Refactor Approach

**CRITICAL: When implementing new features or fixing bugs, you MUST follow the Test Driven Development (TDD) cycle. NO EXCEPTIONS.**

When asked to implement any new functionality, respond with: "I'll follow TDD. First, let me write a failing test for this behavior."

### ENFORCEMENT RULES
- **NEVER write implementation code without a failing test first**
- **MUST write failing tests BEFORE implementing any new functionality**
- **ALWAYS run `cargo test` after each phase to verify state**
- Each feature request requires one complete Red-Green-Refactor cycle

### 1. RED Phase (MANDATORY FIRST STEP)
- Write unit tests for the behavior you are about to implement
- **MUST verify tests fail with `cargo test` before proceeding**
- The tests should fail initially (since the behavior doesn't exist yet)
- Verify that the tests fail for the right reasons
- Focus on what the code should do, not how it should do it
- **Required Rust commands:** `cargo test`, `cargo test --lib`, `cargo test <specific_test>`

### 2. GREEN Phase (ONLY AFTER RED)
- Write the minimal amount of code necessary to make the failing tests pass
- Don't worry about code quality or elegance at this stage
- The goal is simply to make the tests green as quickly as possible
- **MUST verify all tests pass with `cargo test` before proceeding**

### 3. REFACTOR Phase (ONLY AFTER GREEN)
- Improve the code quality while keeping all tests green
- Follow proper coding standards and best practices
- **MUST run `cargo fmt` and `cargo clippy` during this phase**
- Eliminate duplication and improve design
- **MUST run `cargo test` after each refactoring step to ensure nothing breaks**

### WORKFLOW CONSTRAINTS
- If asked to implement feature X, FIRST write a test that demonstrates the missing behavior
- Implementation is only complete when tests pass AND code is refactored
- Each commit should represent one complete Red-Green-Refactor cycle
- Always ask user to confirm test approach before writing implementation
- **NEVER skip any phase of the cycle**

### Rust-Specific Commands (MANDATORY)
- Test execution: `cargo test`
- Code formatting: `cargo fmt`
- Linting: `cargo clippy`
- Build verification: `cargo build`

Remember: **RED → GREEN → REFACTOR → Repeat** (NO SHORTCUTS ALLOWED)