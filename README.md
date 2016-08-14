# kernel

A collaborative effort to rewrite the kernel with focus on correctness and code quality.

## Why?

The kernel code was getting increasingly messy to the point were only the original writer would be able to find and fix bugs. Fortunately, the kernel of Redox is relatively small and such a project is estimated to take only a few months.

## What?

The aims of the new kernel should be clear in their order:

1. **Correctness**: Above anything else, the kernel should be correct. No hacks, despite how the tiny cool shortcuts might seem, it gives severe backslash later on. Keep it correct and well-written.

2. **Readability and documentation**: The code quality should be high, with that follows a detailed documentation, including both API docs (on every item!) and careful comments for anything non-trivial.

3. **Performance**: If you can, go for it.

## Guidelines

### A rotten house is built on a rotten fundament.

Don't fool yourself. You are likely not getting back to the ugly code. Write it the right way **first time**, and make sure you only move on **when it's done right**.

### Comments

Do not hesitate to put comments all over the place.

### Documentation

Every public item should contain API documentation.

### Debug assertions

Abusing debug assertions is a wonderful way to catch bugs, and it is very much encouraged.

### Statical checking

Rust provides a lot of type-system features which can be used to create wonderful safe abstractions, and you should use them whenever you get the chance.

Unsafety should be avoided, and if it is triggered only under some addition **insert an assertion**. Despite this being a kernel, we prefer kernel panics over security vulnarbilities.

If the condition is (or should be) unreachable, but if not upheld, leading to UB, put an assertion in the start of the function.

### Be gentle

Don't just write as much code as you can as quick as possible. Take your time and be careful.

### Commits

Use descriptive commits. One way to force yourself to do that is to not pass the `-m` flag, which will make your editor pop up, so that you can conviniently write long commit messages.
