# Guidelines for AI Coding Agents

Read [docs/SpacetimeDB+SpacetimeDSL.md](docs/SpacetimeDB+SpacetimeDSL.md) to understand how to use SpacetimeDB and SpacetimeDSL.

## Methodologies

### Test Driven Development (TDD)

#### Definitions

| Term                      | Meaning                                                                     |
| ------------------------- | --------------------------------------------------------------------------- |
| **Red**                   | Test compiles, runs, and fails on an assertion                              |
| **Green**                 | Test passes after implementation changes                                    |
| **Refactor**              | Code cleanup while tests remain green                                       |
| **Signature change**      | Adding function parameters, struct fields, enum variants, or variant fields |
| **Implementation change** | Modifying function or method bodies                                         |
| **Observable behavior**   | Outputs, return values, or side effects visible through the public API      |

#### Execution Steps

1. Write a test calling the desired public API.
2. Resolve compilation errors by signature changes only. No implementation changes.
3. Run the test. Confirm assertion failure.
4. Write minimal implementation code.
5. Run the test. Confirm it passes.
6. Refactor. Confirm tests remain green.

#### Phase Validation Criteria

##### Red Phase

Run `cargo test`. All conditions must be true:

| Condition         | Validation                                                     |
| ----------------- | -------------------------------------------------------------- |
| Compiles          | No compilation errors                                          |
| Runs              | Test executes without panic before assertions                  |
| Fails             | Assertion fails (e.g., `assert_eq!` mismatch)                  |
| Signatures only   | Only function signatures, struct fields, enum variants changed |
| Tests behavior    | Assertion checks outputs, not internal state                   |

If any condition is false, fix the test or signatures before proceeding.

| Symptom               | Diagnosis                              | Action                           |
| --------------------- | -------------------------------------- | -------------------------------- |
| Compilation error     | Missing signature elements             | Add parameters, fields, variants |
| Panic before assert   | Missing setup or invalid state         | Fix test setup                   |
| Test passes           | Behavior already exists or test is bad | Verify test logic                |
| Implementation edited | Function body modified                 | Revert implementation changes    |

##### Green Phase

Run `cargo test`. All conditions must be true:

| Condition              | Validation                          |
| ---------------------- | ----------------------------------- |
| Test passes            | Assertion succeeds                  |
| Implementation minimal | No code beyond what test requires   |
| Test unchanged         | Test file has no modifications      |

Modify implementation to pass the test. Keep the test unchanged.

##### Refactor Phase

Run `cargo test` after each change. Condition: all tests pass.

#### Example

Goal: Watcher emits one event per file during rapid saves (currently emits multiple).

##### Red

Step 1: Write test calling desired API.

```rust
#[tokio::test]
async fn test_rapid_saves_produce_single_event() {
    // Arrange
    let config = WatcherConfiguration { debounce_milliseconds: 100 };
    let (mut watcher, mut rx) = create_watcher(config).unwrap();
    watch_path(&mut watcher, &temp_dir).unwrap();

    // Act: trigger multiple filesystem writes
    for i in 0..10 {
        write_file(&path, format!("v{}", i));
    }
    sleep(Duration::from_millis(200)).await;

    // Assert: expect 1 event (will fail when impl added)
    let events: Vec<_> = collect_all(&mut rx);
    assert_eq!(events.len(), 1);
}
```

Step 2: Resolve compilation errors (signature changes only).

```rust
pub struct WatcherConfiguration {
    pub debounce_milliseconds: u64,
}

pub fn create_watcher(config: WatcherConfiguration) -> Result<(Watcher, Receiver)> {
    todo!()  // unchanged body
}
```

Step 3: Run test.

Run: `cargo test test_rapid_saves` → panics on `todo!()` before assertion → Fix: replace `todo!()` with minimal non-panicking stub → Run again → fails with `assertion failed: 1 != 10` → Red confirmed.

##### Green

Implement deduplication using `config.debounce_milliseconds` in `create_watcher`. Run: `cargo test test_rapid_saves` → passes → Green confirmed.

##### Refactor

Clean up. Run: `cargo test` → all pass → Refactor complete.

### Self-Documenting Code

Code must be self-documenting through clear naming:

- **No abbreviations**: Use `InputOutput` not `Io`, `FileSystemWatcher` not `Watcher`, `DirectoryWalker` not `Walk`.
- **Descriptive identifiers**: Names should convey meaning without requiring comments.
- **No redundant comments**: Never document "how" - the code shows that. Only document "what" and "why" when not obvious from the code itself.
- **Remove comments that repeat the code**: A comment like `/// IO error.` above `Io(io::Error)` adds no value.

## Programming Principles

### Principle Checklists

Use the following bullet points as checklist when planning, reviewing or implementing code changes.

#### Scope & Goal Discipline

- [ ] Identify essential outcome before touching code.
- [ ] Remove nonessential requirements, defer speculative work.
- [ ] Work limited to current user story.
- [ ] Future ideas captured outside codebase.
- [ ] Latest change solves current requirement only.

#### Simplicity & Right-Sized Solutions

- [ ] Prefer straightforward data flows over clever abstractions.
- [ ] Review changes for simpler alternatives prior merge.
- [ ] Defer abstractions until duplication appears.
- [ ] Prefer linear solution before optimizing structures.
- [ ] Explain why added complexity was unavoidable.
- [ ] Delay abstractions until duplication patterns stay consistent.

#### Documentation & Communication Clarity

- [ ] Document validation steps before implementation begins.
- [ ] Document concern boundaries and integration contracts.
- [ ] Document intent so future maintainers understand choices.
- [ ] Prefer clarity over cleverness in code and comments.
- [ ] Apply Principle of Least Astonishment in interfaces.
- [ ] Document extension points so the framework knows when to call custom logic.
- [ ] Log blockers to future cleanups for retrospectives.

#### Refactoring & Change Containment

- [ ] Unused scaffolding removed immediately.
- [ ] Refactoring kept change radius shallow.
- [ ] Refactor immediately once simplest version works.
- [ ] Leave touched files cleaner than before commit.
- [ ] Fix small clarity issues during adjacent work.
- [ ] Stop when improvements risk derailing feature.

#### Testing & Verification

- [ ] Added tests justify each new branch.
- [ ] Reference user-impacting behaviors with guardrails or tests.
- [ ] Add tests before refactoring or when missing.
- [ ] Add tests for coupling regressions immediately.
- [ ] Keep unit tests running in milliseconds.
- [ ] Ensure each test has no shared state.
- [ ] Stabilize tests across environments and runs.
- [ ] Make assertions binary without manual inspection.
- [ ] Write tests before production code.
- [ ] Arrange inputs and environment first.
- [ ] Act with one focused call.
- [ ] Assert clear outcomes and side effects.

#### Modular Boundaries & Separation

- [ ] Split work into minimally overlapping modules.
- [ ] User interface changes shouldn't touch domain logic.
- [ ] Shared utilities expose APIs, not internal details.
- [ ] Keep unrelated responsibilities decoupled across modules.
- [ ] One domain change updates just one system surface.
- [ ] Design APIs without cross-cutting side effects.
- [ ] Prefer composable primitives over special-case hooks.
- [ ] Localize domain changes to the smallest module set.
- [ ] Avoid edits that force neighboring modules to change.
- [ ] Keep persistence, formatting, business logic in separate classes.

#### Coupling Awareness & Dependency Constraints

- [ ] Evaluate coupling via strength, locality, degree axes.
- [ ] Prefer weaker connascence forms when refactoring dependencies.
- [ ] Keep high-degree dependencies co-located or abstracted.
- [ ] Discuss coupling issues using shared connascence vocabulary.
- [ ] Reduce remote connascence before crossing service boundaries.
- [ ] Limit cross-module dependencies before coding integrations.
- [ ] Remove shared global state unless absolutely justified.
- [ ] Keep method calls within Law of Demeter bounds.
- [ ] Review change blast radius for every edit.
- [ ] Anticipate maintenance; avoid hidden coupling or magic.

#### Duplication Control & Reuse

- [ ] One authoritative source for each business rule.
- [ ] Abstract repeated logic instead of copy/paste fixes.
- [ ] Sync related artifacts—code, docs, tests—whenever knowledge changes.
- [ ] Remove duplication without coupling unrelated responsibilities.

#### Dependency & Interface Management

- [ ] Decide early which responsibilities belong to the framework versus custom modules.
- [ ] Route orchestration through containers, factories, or callbacks instead of ad-hoc callers.
- [ ] Depend on contracts/interfaces so modules stay decoupled from specific implementations.
- [ ] Provide dependencies via injection or lookup rather than instantiating collaborators internally.
- [ ] Decouple high-level modules to maximize reuse and maintainability.
- [ ] Inject abstractions so mocks enable isolated unit tests.
- [ ] Shield changes to reduce cascading failures.
- [ ] Introduce new implementations without touching existing clients.
- [ ] Prefer stable abstractions over volatile concretions.
- [ ] Define interfaces to represent dependencies.
- [ ] Wire classes against abstractions instead of concretions.
- [ ] Adopt DI patterns when selecting collaborators.
- [ ] Leverage IoC containers to manage lifecycle ownership.
- [ ] Keep systems decoupled to ease refactors, changes, redeployments.
- [ ] Split fat interfaces so clients receive only needed methods.
- [ ] Reject requirements that force SRP-violating interface methods.

#### Robustness & Reliability

- [ ] Enforce strict output formats before sending responses.
- [ ] Accept unknown inputs only when semantics remain clear.
- [ ] Log and surface malformed partner payloads immediately.
- [ ] Document tolerance rules plus removal plans for shims.
- [ ] Prefer protocol fixes over bug-for-bug compatibility.

#### Performance & Optimization Discipline

- [ ] Profile hotspots before considering any micro-optimization.
- [ ] Define performance target and acceptable baseline early.
- [ ] Optimize only after failing measurable acceptance criteria.
- [ ] Preserve readability; document every performance tradeoff.
- [ ] Rerun regression and performance tests after tuning.

#### Lifecycle & Deletion Strategy

- [ ] Keep modules small enough to rewrite in a week.
- [ ] Limit coupling so deletions don't require cascade edits.
- [ ] Isolate features so removal doesn't break shared contracts.
- [ ] Record explicit dependencies to spot safe deletion seams.
- [ ] Delete code, tests, and configuration in the same pass.

#### Cohesion & Responsibility Alignment

- [ ] Keep each module focused on one responsibility.
- [ ] Cut scope until module complexity stays low.
- [ ] Group related operations so components stay reusable.
- [ ] Localize each change request to one module.
- [ ] Split code by stakeholder or actor-specific responsibilities.
- [ ] Apply Curly's Law so classes do one job.
- [ ] Extract new classes whenever unrelated change reasons emerge.
- [ ] Define the single goal before touching a code path.
- [ ] Reject extra responsibilities that dilute that one outcome.
- [ ] Align module boundaries with a single user-visible outcome.
- [ ] Split methods/classes until each has one change driver.
- [ ] Explicitly state what the unit will not cover.

#### Encapsulation & Interface Hygiene

- [ ] Limit calls to immediate collaborators only.
- [ ] Avoid chaining through returned collaborators.
- [ ] Push delegation into owning object interfaces.
- [ ] Expose DTOs for views instead of domain graphs.
- [ ] Document justified exceptions when structure must stay public.
- [ ] Hide implementation details behind small, stable interfaces.
- [ ] Stabilize interfaces so client changes stay unnecessary.
- [ ] Minimize class and member accessibility.
- [ ] Keep member data private and encapsulated.
- [ ] Exclude private implementation details from public interfaces.
- [ ] Reduce coupling to conceal implementation details.

#### Composition & Object Design

- [ ] Validate relationship is has-a before inheriting.
- [ ] Split behaviors into small interfaces or components.
- [ ] Use delegation to avoid breaking LSP.
- [ ] Allow runtime swapping of composed collaborators.
- [ ] Document extra obligations when inheritance unavoidable.
- [ ] Subclasses honor every pre/postcondition promised by supertype.
- [ ] Never strengthen preconditions when overriding base behavior.
- [ ] Never weaken postconditions or invariants in derived classes.
- [ ] Subtypes only throw exceptions declared by the base.
- [ ] Remove hierarchies that force instanceof checks in clients.

#### Variation Isolation & Extensibility

- [ ] Minimize changes to existing code to preserve stability.
- [ ] Favor extension points over direct modification.
- [ ] Hide non-variant details and expose only adjustable seams.
- [ ] Minimize edits whenever change happens.
- [ ] Hide each varying concept behind interface.
- [ ] Isolate varying concept in standalone module.

#### Command/Query Interaction Design

- [ ] Separate queries (reading) from commands (writing) to boost confidence.
- [ ] Declare each method solely query or command.
- [ ] Name methods to signal query versus command behavior.

### Principles

#### Keep It Simple, Stupid (KISS)

_Checklist coverage:_ Scope & Goal Discipline; Simplicity & Right-Sized Solutions; Documentation & Communication Clarity.

#### You Aren't Gonna Need It (YAGNI)

_Checklist coverage:_ Scope & Goal Discipline; Refactoring & Change Containment; Testing & Verification.

#### Do The Simplest Thing That Could Possibly Work

_Checklist coverage:_ Scope & Goal Discipline; Simplicity & Right-Sized Solutions; Refactoring & Change Containment.

#### Separation of Concerns

_Checklist coverage:_ Documentation & Communication Clarity; Modular Boundaries & Separation.

#### Code For The Maintainer

_Checklist coverage:_ Documentation & Communication Clarity; Testing & Verification; Coupling Awareness & Dependency Constraints.

#### Avoid Premature Optimization

_Checklist coverage:_ Performance & Optimization Discipline.

#### Optimize for Deletion

_Checklist coverage:_ Lifecycle & Deletion Strategy.

#### Don't Repeat Yourself (DRY)

_Checklist coverage:_ Simplicity & Right-Sized Solutions; Duplication Control & Reuse.

#### Boy Scout Rule

_Checklist coverage:_ Documentation & Communication Clarity; Refactoring & Change Containment; Testing & Verification.

#### Connascence

_Checklist coverage:_ Coupling Awareness & Dependency Constraints.

#### Minimize Coupling

_Checklist coverage:_ Coupling Awareness & Dependency Constraints; Encapsulation & Interface Hygiene.

#### Law of Demeter

_Checklist coverage:_ Encapsulation & Interface Hygiene.

#### Composition Over Inheritance

_Checklist coverage:_ Composition & Object Design.

#### Orthogonality

_Checklist coverage:_ Modular Boundaries & Separation; Testing & Verification.

#### Robustness Principle

_Checklist coverage:_ Robustness & Reliability.

#### Inversion of Control

_Checklist coverage:_ Documentation & Communication Clarity; Dependency & Interface Management.

#### Maximize Cohesion

_Checklist coverage:_ Modular Boundaries & Separation; Cohesion & Responsibility Alignment.

#### Liskov Substitution Principle (LSP)

_Checklist coverage:_ Composition & Object Design.

#### Open/Closed

_Checklist coverage:_ Variation Isolation & Extensibility.

#### Single Responsibility Principle (SRP)

_Checklist coverage:_ Modular Boundaries & Separation; Cohesion & Responsibility Alignment.

#### Hide Implementation Details

_Checklist coverage:_ Encapsulation & Interface Hygiene.

#### Curly's Law

_Checklist coverage:_ Cohesion & Responsibility Alignment.

#### Encapsulate What Changes

_Checklist coverage:_ Variation Isolation & Extensibility.

#### Interface Segregation Principle (ISP)

_Checklist coverage:_ Dependency & Interface Management.

#### Command Query Separation (CQS)

_Checklist coverage:_ Command/Query Interaction Design.

#### Dependency Inversion Principle (DIP)

_Checklist coverage:_ Dependency & Interface Management.

#### F.I.R.S.T Principles of Testing

_Checklist coverage:_ Testing & Verification.

#### Arrange, Act, Assert (3A)

_Checklist coverage:_ Testing & Verification.

### Conflicts between Programming Principles

- **You Aren't Gonna Need It (YAGNI) vs Boy Scout Rule** — Cleaning adjacent code “just because” risks shipping scope beyond the current story YAGNI enforces, while only delivering the current story can stop you from making opportunistic cleanups the Boy Scout Rule encourages.
  - **Important Action**: Favor the "Boy Scout Rule" over the "You Aren't Gonna Need It (YAGNI)" principle.
- **Separation of Concerns vs Maximize Cohesion** — Splitting behavior strictly by technical concern can scatter workflows that cohesion would otherwise keep together, while grouping everything by cohesive domain can blur the technical boundaries the Separation of Concerns checklist enforces.
  - **Important Action**: Favor the "Maximize Cohesion" principle without violating the "Separation of Concerns" principle by splitting in domain modules which wire together technical modules.
- **Code For The Maintainer vs Optimize for Deletion** — Designing code to be disposable may trade away the documentation and guardrails maintainers need, while preserving extensive context and commentary makes code heavier and harder to throw away.
  - **Important Action**: Favor the "Optimize for Deletion" principle without violating the "Code For The Maintainer" principle by writing self-documenting code.
- **Code For The Maintainer vs Hide Implementation Details** — Strict encapsulation can obscure the intent and rationale that future maintainers are supposed to see, yet surfacing intent for future readers can pressure you to expose structure that this principle would otherwise hide.
  - **Important Action**: Favor the "Hide Implementation Details" principle without violating the "Code For The Maintainer" principle by maintaining clear conventions for data flow from entry points to utilities.
- **Avoid Premature Optimization vs Optimize for Deletion** — Engineering for easy removal ahead of evidence is still speculative optimization before you know deletion is necessary, while the directive to defer speculative work pushes back on adding deletion seams before they are justified.
  - **Important Action**: Favor the "Optimize for Deletion" principle because the "Avoid Premature Optimization" principle is about performance (CPU cycles, memory usage, etc.), which is more measurable than code quality.
- **Don't Repeat Yourself (DRY) vs Interface Segregation Principle (ISP)** — Splitting interfaces per client can reintroduce similar method signatures, undermining a single authoritative implementation, but centralizing behavior for reuse can bloat interfaces with methods some clients do not need.
  - **Important Action**: Favor the "Interface Segregation Principle" principle without violating the "Don't Repeat Yourself" principle by carefully managing shared abstractions used by the different client interfaces.
- **Connascence vs Orthogonality** — Enforcing total independence between modules can make it harder to surface and manage the necessary relationships connascence tracks, whereas allowing related elements to evolve together acknowledges coupling that orthogonality tries to eliminate.
  - **Important Action**: Favor the "Connascence" principle without violating the "Orthogonality" principle by splitting in domain modules which wire together technical modules.
- **Minimize Coupling vs Command Query Separation (CQS)** — Splitting reads and writes across distinct interfaces increases the number of collaborators you have to coordinate, yet reducing the number of dependencies can discourage the extra command/query partitions this principle demands.
  - **Important Action**: Favor the "Command Query Separation" principle over the "Minimize Coupling" principle.
- **Law of Demeter vs Inversion of Control** — Depending on global containers or callbacks can force knowledge of collaborators beyond the immediate neighbor, and IoC plumbing may require reaching into containers or nested delegates, effectively talking to strangers' collaborators.
  - **Important Action**: Favor the "Law of Demeter" principle over the "Inversion of Control" principle.
- **Composition Over Inheritance vs Liskov Substitution Principle (LSP)** — Favoring composition can mean rejecting subtype hierarchies even when safe substitution could simplify clients, while insisting on composition can prevent you from leveraging polymorphic substitution even when it preserves contracts.
  - **Important Action**: Favor the "Composition Over Inheritance" principle over the "Liskov Substitution Principle (LSP)" principle.
- **Maximize Cohesion vs Single Responsibility Principle (SRP)** — Keeping all closely-related actions together may introduce multiple reasons for the same module to change, and keeping every related behavior together can mix multiple reasons for change inside a single module.
  - **Important Action**: Favor the "Maximize Cohesion" principle without violating the "Single Responsibility Principle (SRP)" principle by splitting in domain modules which wire together technical modules.
- **Curly's Law vs Encapsulate What Changes** — Focusing on isolating volatility can slice one user-facing outcome across many modules, violating the single-goal guidance, while keeping all logic for one goal together can make it harder to extract the unstable parts into their own modules.
  - **Important Action**: Favor "Curly's Law" without violating the "Encapsulate What Changes" principle by splitting in domain modules which wire together technical modules.

- **Keep It Simple, Stupid (KISS) vs Open/Closed** — Designing extension seams up front can add abstraction layers that move the system away from the simplest implementation, while stripping code down to its bare minimum can remove the extension points the Open/Closed principle expects.
  - **Action**: Favor the "Keep It Simple, Stupid (KISS)" principle over the "Open/Closed" principle.
- **You Aren't Gonna Need It (YAGNI) vs Robustness Principle** — Hardening for malformed inputs may require building capabilities no stakeholder has requested yet, but skipping defensive code for future scenarios contradicts the robustness expectation to handle unknown inputs now.
  - **Action**: Favor the "Robustness Principle" over the "You Aren't Gonna Need It (YAGNI)" principle.
- **You Aren't Gonna Need It (YAGNI) vs F.I.R.S.T Principles of Testing** — Authoring comprehensive, pre-emptive tests can feel like upfront work outside the immediate requirement, yet limiting work to what is requested can undercut the up-front comprehensive coverage that F.I.R.S.T expects.
  - **Action**: Favor the "F.I.R.S.T Principles of Testing" over the "You Aren't Gonna Need It (YAGNI)" principle.
- **Do The Simplest Thing That Could Possibly Work vs Dependency Inversion Principle (DIP)** — Introducing abstractions and injection layers adds ceremony beyond the quickest working solution, while pursuing the most direct implementation can skip the abstraction seams that dependency inversion requires.
  - **Action**: Favor the "Dependency Inversion Principle (DIP)" over the "Do The Simplest Thing That Could Possibly Work" principle.
- **Do The Simplest Thing That Could Possibly Work vs Inversion of Control** — Delegating orchestration to containers or callbacks introduces indirection that contradicts the most straightforward implementation, yet wiring everything through IoC containers is rarely the most straightforward path for a small change.
  - **Action**: Favor the "Inversion of Control" principle over the "Do The Simplest Thing That Could Possibly Work" principle.
- **Separation of Concerns vs Arrange, Act, Assert (3A)** — Keeping setup, action, and verification steps co-located inside one test clashes with attempts to isolate each concern into shared helpers, but forcing those steps into separate fixtures undermines the single-test narrative the Arrange, Act, Assert pattern emphasizes.
  - **Action**: Favor the "Separation of Concerns" principle over the "Arrange, Act, Assert (3A)" principle.
