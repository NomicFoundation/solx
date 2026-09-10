---
name: review
description: Read-only code review against CLAUDE.md. Ambiguities reach the user as questions, then every finding as a report. Nothing is changed. Use when the code builds and passes its tests.
---

# Review

The skill changes nothing. It reads, checks, asks, and reports.

## What is reviewed

What the user names when invoking: a diff range, a branch, a path or a file. If nothing is named, the current branch against `main`. If the branch is `main`, the uncommitted changes. If there are none, ask the user with AskUserQuestion what to review.

## Readers

Launch a reader per lens, on Opus unless the invocation names another model. Each reader reads this repository's [`CLAUDE.md`](../../../CLAUDE.md) in full, then the whole change, and asks its question of every clause the change adds or alters.

1. Removal: what fails if this clause is deleted? A clause nothing fails without is the finding.
2. Trust: what does this clause check, derive or own that a lower layer already guarantees? The guarantee, read in that layer's code, is the evidence.
3. Foreignness: where does the repository already do this another way? The sibling that does it is the evidence.
4. Pinning: which claim of the change has no test that fails when the code is wrong? The mutation that leaves the test green is the evidence.

The invocation may add lenses. Each added lens is its own question and its own reader.

A reader runs nothing that writes, and reproduces a claim only on a scratch copy. It reports each finding with the file and line, what is wrong, the evidence, and the fix. A finding without evidence is not a finding.

## Checking

Every finding is re-read against the code before it goes further. One whose evidence does not hold is dropped. The same finding from several readers is reported once.

## Decisions

A decision is anything the review cannot settle from the code and `CLAUDE.md`: a finding with more than one defensible fix, a rule that reads two ways for this change, or a change whose intent the diff does not show. Each decision is put to the user with AskUserQuestion, one at a time, every option naming the evidence for it. The user's answer is final.

## Report

After the last decision is answered, every finding that survived checking, most consequential first. Each is a short paragraph: what is wrong, the evidence, what to do, and for a decided finding, the user's answer. Nothing is applied.
