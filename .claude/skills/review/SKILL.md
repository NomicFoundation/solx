---
name: review
description: Read-only code review against CLAUDE.md. Ambiguities reach the user as questions, then every finding as a report. No code is changed, and the report goes on the pull request only when the user asks. Use when the code builds and passes its tests.
---

# Review

The skill changes no code. It reads, checks, asks, reports, and posts when the user asks.

Every question prompts the user with a fixed set of options, never free prose: AskUserQuestion under Claude Code, its equivalent under another harness. Readers run on the model the invocation names, otherwise Opus under Claude Code and the configured model elsewhere.

## What is reviewed

What the user names when invoking: a diff range, a branch, a path or a file. If nothing is named, in order:

- `<base>...HEAD`, where `<base>` is the branch the pull request targets.
- If `HEAD` is that base branch, the uncommitted changes.
- If there are none, prompt the user for what to review.

## Readers

Launch a reader per lens. Each reader reads this repository's [`CLAUDE.md`](../../../CLAUDE.md) in full, then the whole change, and asks its question of every clause the change adds or alters.

1. Removal: what fails if this clause is deleted? A clause nothing fails without is the finding.
2. Trust: what does this clause check, derive or own that a lower layer already guarantees? The guarantee, read in that layer's code, is the evidence.
3. Foreignness: where does the repository already do this another way? The sibling that does it is the evidence.
4. Pinning: which claim of the change has no test that fails when the code is wrong? The mutation that leaves the test green is the evidence.

The invocation may add lenses. Each added lens is its own question and its own reader.

A reader writes nothing in the repository. Pinning argues the mutation from the test, and may run it on a scratch copy.

It reports each finding with:

- the file and line
- the rule of `CLAUDE.md` it rests on, by section and number
- what is wrong
- the evidence
- the fix

A finding without evidence is not a finding. A finding without a rule is either a rule `CLAUDE.md` is missing or not a finding.

## Checking

Every finding is re-read against the code before it goes further.

- One whose evidence does not hold is dropped.
- One whose fix a repository lint would undo does not hold either.
- The same finding from several readers is reported once.

## Decisions

A decision is anything the review cannot settle from the code and `CLAUDE.md`:

- a finding with more than one defensible fix
- a rule that reads two ways for this change
- a change whose intent the diff does not show

Each decision prompts the user, one at a time, every option naming the evidence for it. The user's answer is final.

- Decisions are deduped like findings.
- A design-related decision carries a proposal to add the rule to `CLAUDE.md`.

## Report

After the last decision is answered, every finding that survived checking, most consequential first. Each is a short paragraph:

- the rule it rests on
- what is wrong
- the evidence
- what to do
- for a decided finding, the user's answer

## Posting

The report stays in the terminal. It goes on the pull request when the user asks for it.

### Inline comments

- A finding becomes a comment on every line it holds at, opening with one sentence saying what is wrong.
- GitHub accepts an anchor only inside a diff hunk, so check every anchor against the hunk ranges before submitting.
- A finding about code the diff does not touch goes on the diff line whose correctness depends on it, and names the code it is really about.

### Review comment

- The findings with no line of their own.
- The design proposals.
- The proposed changes to `CLAUDE.md` and the documentation.
