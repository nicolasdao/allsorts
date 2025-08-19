## 1. Preparation

1. **Read Documentation**

   * Review `README.md` (or your project’s equivalent) to understand overall architecture.
   * Read the feature specification (e.g. `specs/<date>_<feature>/execute.md`).

2. **Verify Test Health**

   ```bash
   # Run existing tests
   <run-tests>    # e.g. `npm test`, `pytest`, `mvn test`, `cargo test`
   ```

   * If any tests fail:

     1. **Stop.**
     2. Fix existing failures.
     3. Re-run until the suite is green.

---

## 2. Analysis & Planning

1. **High-Level Design**

   * Summarize the goal: “Add support for `<feature>`.”
   * Identify affected modules, components, and external dependencies.

2. **Break Down Into Tasks**

   * List sub-features or components (e.g. parsing, serialization, UI, error handling).
   * Sketch interactions and data flow between them.

3. **Edge-Case Brainstorm**

   * Enumerate valid inputs, boundary conditions, error scenarios, performance constraints, etc.

---

## 3. Define APIs & Signatures

For each sub-task, draft the interface without implementation:

| Component / Function   | Inputs                         | Outputs                     | Error Cases / Behaviors   |
| ---------------------- | ------------------------------ | --------------------------- | ------------------------- |
| `<Module>.parseData()` | `raw: <type>`                  | `<ParsedType>` or `<Error>` | Handles malformed input   |
| `<Module>.serialize()` | `obj: <Type>, options: <Opts>` | `<byte[]>` or `<Error>`     | Applies compression rules |
| …                      | …                              | …                           | …                         |

> **Note:** Only signatures and documentation—no implementation yet.

---

## 4. Write Unit Tests (TDD)

1. **Set Up Test Files**

   * Create or update test files under your test directory.
   * Follow your project’s naming conventions.

2. **Specify Test Cases**

   * **Happy path(s):** correct input → expected output.
   * **Edge cases:** empty/minimal data, boundary values.
   * **Error cases:** invalid inputs → specific errors or exceptions.

3. **Isolate Tests**

   * Each test targets a single function or behavior.
   * Tests should compile/load but **fail** initially (since implementations don’t exist yet).

```
// Example (pseudocode)
test("parseData rejects empty input", () => {
  expect(() => parseData("")).toThrow(ParseError.EmptyInput);
});
```

---

## 5. Implement Functions Incrementally

For each function or module:

1. **Minimal Implementation**

   * Write only enough code to satisfy one failing test.

2. **Run Specific Tests**

   ```bash
   <run-tests> --filter <test-name>
   ```

3. **Iterate**

   * If tests still fail, refine code until they pass.
   * Proceed to the next test case or function.

4. **Repeat**

   * Continue until *all* tests for this component pass.

---

## 6. Full-Suite Integration

1. **Execute Entire Test Suite**

   ```bash
   <run-tests>
   ```

2. **Fix Regressions**

   * Address any failures—both new and existing.

---

## 7. Documentation & Review

1. **Update Documentation**

   * Revise user guides, API docs, changelog, etc.

2. **Ensure Code Comments**

   * Describe public interfaces and complex logic.

3. **Peer Review**

   * Open a pull request / merge request and request feedback.
   * Address comments and suggestions.

4. **Merge & Release**

   * Tag the release version.
   * Notify stakeholders or publish release notes.

---

### 🗂️ Checklist

* [ ] Existing tests green before starting
* [ ] Interfaces & signatures defined
* [ ] Unit tests created (initially failing)
* [ ] Incremental implementation of functions
* [ ] Full test suite passing
* [ ] Documentation updated
* [ ] Reviewed and merged
