# Contributing to Redox

_**Thank you for your interest in contributing to Redox!** This document will outline the basics of where to start if you wish to contribute to the project. There are many ways to help us out and and we appreciate all of them. We look forward to **your contribution**!_

## Index

* [Communication](#communication)
 * [Chat](#chat)
 * [GitLab Issues](#gitlab-issues)
 * [Pull Requests](#pull-requests)
* [Best Practices/Guidelines](#best-practices-and-guidelines)
 * [General](#general)
 * [Testing Practices](#testing-practices)
* [Style Guidelines](#style-guidelines)
 * [Rust](#rust)
 * [Git](#git-style-guidelines)
* [Other Ways to Contribute](#other-ways-to-contribute)
 * [Design](#design)

## Other External Links

* [redox-os.org](https://redox-os.org)
* [rust-os-comparison](https://github.com/flosse/rust-os-comparison)
* [rust-lang.org](http://rust-lang.org)

## Communication

### Chat

- [Redox Dev room]
- [Redox Support room]

[Redox Dev room]: https://matrix.to/#/#redox-dev:matrix.org
[Redox Support room]: https://matrix.to/#/#redox-support:matrix.org

### GitLab Issues

A bit more formal way of communication with fellow Redox devs, but a little less quick and convenient like the chat. Submit an issue when you run into problems compiling, testing, or just would like to discuss a certain topic, be it _features, code style, code inconsistencies, minor changes and fixes, etc._

### Pull Requests

[How to make pull requests properly]: https://doc.redox-os.org/book/ch12-04-creating-proper-pull-requests.html

## Best Practices and Guidelines

### General

* **Remember to do a `git rebase -i upstream/master` before you send your patch!**
* **Make sure your code is readable, commented, and well-documented.**
* **Don't hesitate to ask for help, comments or suggestions!**
* **Before implementing something, discuss it! Open an issue, or ask in the chat.**

### Testing Practices

* It's always better to test boot (`make qemu` or `make virtualbox`) every time you make a change, because it is important to see how the OS boots and works after it compiles.
  Even though Rust is a safety-oriented language, something as unstable and low-level as an in-dev operating system will almost certainly have problems in many cases and may completely break on even the slightest critical change.
  Also, make sure you check how the unmodified version runs on your machine before making any changes. Else, you won't have anything to compare to, and it will generally just lead to confusion. TLDR: Rebuild and test boot often.

## Style Guidelines

### Rust

Since **Rust** is a relatively small and new language compared to others like _C_, there's really only one standard. Just follow the official Rust standards for formatting, and maybe run `rustfmt` on your changes, until we setup the CI system to do it automatically.

### Git

* You should have a fork of the repository on **GitLab** and a local copy on your computer. The local copy should have two remotes; `origin` and `fork`, `origin` should be set to the main repository and `fork` should be your fork.
* When you start to make changes, you will want to create a separate branch, and keep the `master` branch of your fork identical to the main repository, so that you can compare your changes with the main branch and test out a more stable build if you need to.
* Usually, when syncing your local copy with the master branch, you'll want to rebase instead of merge. This is because it will create duplicate commits that don't actually do anything when merged into the master branch. You can do this in one command with `git pull origin --rebase`. This will pull from the upstream, then roll back to the current state of the upstream, and "replay" your changes on top of it. Make sure you commit before doing this, though. Git won't be able to rebase if you don't.
* Prefer to omit the `-m` when using `git commit`. This opens your editor and should help get you in the habit of writing longer commit messages.
* Commit messages should describe their changes in present tense, e.g. "`Add stuff to file.ext`" instead of "`added stuff to file.ext`". This makes sense as sometimes when you revert back, then run through commits one-by-one, you want to see what a commit will do, instead of just what the person did when they made the commit. It's also just being consistent.
* Try to remove useless duplicate/merge commits from PRs as these don't do anything except clutter up history and make it harder to read.

## Other Ways to Contribute

If you're not big on coding, but you still want to help keep the project going, you can still contribute/support in a variety of ways! We'll try to find a way to use anything you have to offer. 

### Design

If you're a good designer, whether it's _2D graphics, 3D graphics, interfaces, web design, you can help. We need logos, UI design, UI skins, app icons, desktop backgrounds, etc_. More information to come on this in the future, for now just join [the chat](#chat) and ask about graphic design.
