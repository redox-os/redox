# Contributing to Redox

Thank you for your interest in contributing to Redox! This document is a guide to help newcomers contribute!  
There are many ways to help us out and we appreciate all of them.

## Index

* [Communication](#communication)
 * [Chat](#chat)
 * [Reddit](#reddit)
* [Direct Contributing](#direct-contributing)
 * [Low-Hanging Fruit - Easy Targets for Newbies](#easy-targets)
 * [GitHub Issues](#gh-issues)
 * [Pull Requests](#prs)
 * [Creating a Pull Request](#creating-a-pr)
* [Best Practices/Guidelines](#best-practices)
 * [General](#general)
 * [Kernel](#kernel)
 * [Testing Practices](#testing-practices)
* [Style Guidelines](#style-guidelines)
 * [Rust](#rust-style-guidelines)
 * [Git](#git-style-guidelines)
* [Other Ways to Contribute](#other)
 * [Graphic Design](#graphic-design)

## <a name="extern-links"> Other External Links </a>

* [redox-os.org](http://redox-os.org)
* [rust-os-comparison](https://github.com/flosse/rust-os-comparison)
* [rust-lang.org](http://rust-lang.org)

## <a name="communication"> Communication </a>

### <a name="chat"> Chat </a>

The quickest and most open way to communicate with the Redox team is on our chat server. Currently, the only way to join it is by sending an email to [info@redox-os.org](mailto:info@redox-os.org), which might take a little while, since it's not automated. We're currently working on an easier way to do this, but this is the most convenient way right now.

### <a name="reddit"> Reddit </a>

You can find Redox on Reddit in [/r/rust/](https://www.reddit.com/r/rust) and [/r/redox/](https://www.reddit.com/r/redox). The weekly update news is posted on the former.

## <a name="direct-contributing"> Direct Contributing </a>

### <a name="easy-targets"> Low-Hanging Fruit - Easy Targets for Newbies </a>

* If you're not fluent in Rust:

 * Writing documentation
 * Using/testing Redox, filing issues for bugs and needed features
 * Web development ([Redox website, separate repo](https://github.com/redox-os/website))
 * Writing unit tests (may require minimal knowledge of rust)

* If you are fluent in Rust, but not OS Development:

 * Apps development
 * Shell ([Ion](https://github.com/redox-os/ion)) development
 * Package manager ([Magnet](https://github.com/redox-os/magnet)) development
 * Other high-level code tasks

* If you are fluent in Rust, and have experience with OS Dev:

 * Familiarize yourself with the repository and codebase
 * Grep for `TODO`, `FIXME`, `BUG`, `UNOPTIMIZED`, `REWRITEME`, `DOCME`, and `PRETTYFYME` and fix the code you find.
 * Improve and optimize code, especially in the kernel

### <a name="gh-issues"> GitHub Issues </a>

A bit more formal way of communication with fellow Redox devs, but a little less quick and convenient like the chat (unless of course you aren't in it yet, which if you're going to be involved in this project really at all, it is recommended that you request to join). These are for more specific topics.

### <a name="prs"> Pull Requests </a>

It's completely okay to just submit a small pull request without first making an issue or something, but if it's a significant change that will require a lot of planning and reviewing, it's best you start with writing an issue first. Also see [git guidelines](#git-style-guidelines)

### <a name="creating-a-pr"> Creating a Pull Request </a>

1. Fork the repository
2. Clone the original repository to your local PC using one of the following commands based on the protocol you are using:
 * HTTPS:`git clone https://github.com/redox-os/redox.git --origin upstream --recursive`
 * SSH:`git clone git@github.com:redox-os/redox.git --origin upstream --recursive`
 * Then rebase: `git rebase upstream master`  
 Use HTTPS if you don't know which one to use. (Recommended: learn about SSH if you don't want to have to log in every time you push/pull!)
3. Add your fork with
 * HTTPS:`git remote add origin https://github.com/your-username/redox.git`
 * SSH:`git remote add origin git@github.com:your-username/redox.git`
4. Alternatively, if you already have a fork and copy of the repo, you can simply check to make sure you're up-to-date
 * Fetch the upstream:`git fetch upstream master`
 * Rebase with local commits:`git rebase upstream/master`
 * Update the submodules:`git submodule update --init`
5. Optionally create a separate branch (recommended if you're making multiple changes simultaneously) (`git checkout -b my-branch`)
6. Make changes
7. Commit (`git add . --all; git commit -m "my commit"`)
8. Optionally run [rustfmt](https://github.com/rust-lang-nursery/rustfmt) on the files you changed and commit again if it did anything (check with `git diff` first)
9. Test your changes with `make qemu` or `make virtualbox` (you might have to use `make qemu kvm=no`, formerly `make qemu_no_kvm`)
(see [Best Practices and Guidelines](#best-practices))
10. Pull from upstream (`git fetch upstream; git rebase upstream/master`) (Note: try not to use `git pull`, it is equivalent to doing `git fetch upstream; git merge master upstream/master`, which is not usually preferred for local/fork repositories, although it is fine in some cases.)
11. Repeat step 9 to make sure the rebase still builds and starts
12. Push to your fork (`git push origin my-branch`)
13. Create a pull request
14. Describe your changes
15. Submit!

## <a name="best-practices"> Best Practices and Guidelines </a>

### <a name="general"> General </a>

* **Remember to do a `git rebase -i upstream/master` before you send your patch!**
* **Make sure your code is readable, commented, and well-documented.**
* **Don't hesitate to ask for help!**
* **Before implementing something, discuss it! Open an issue, or join the chat.**

##### On the more technical side:
* Test, test, and test!
* Follow the style conventions
* Use `std::mem::replace` and `std::mem::swap` when you can.
* `libredox` should be 1-to-1 with the official `libstd`.
* Use `.into()` and `.to_owned()` over `.to_string()`.
* Prefer passing references to the data over owned data. (Don't take `String`, take `&str`. Don't take `Vec<T>` take `&[T]`).
* Use generics, traits, and other abstractions Rust provides.
* Avoid using lossy conversions (for example: don't do `my_u32 as u16 == my_u16`, prefer `my_u32 == my_u16 as my_u32`).
* Prefer in place (`box` keyword) when doing heap allocations.
* Prefer platform independently sized integer over pointer sized integer (`u32` over `usize`, for example).
* Follow the usual idioms of programming, such as "composition over inheritance", "let your program be divided in smaller pieces", and "resource acquisition is initialization".
* When `unsafe` is unnecessary, don't use it. 10 lines longer safe code is better than more compact unsafe code!
* Be sure to mark parts that need work with `TODO`, `FIXME`, `BUG`, `UNOPTIMIZED`, `REWRITEME`, `DOCME`, and `PRETTYFYME`.
* Use the compiler hint attributes, such as `#[inline]`, `#[cold]`, etc. when it makes sense to do so.
* Check the [chat](#chat), [the Website](http://redox-os.org), and [the Rust subreddit](https://www.reddit.com/r/rust) frequently.

### <a name="kernel"> Kernel </a>

* When trying to access a slice, **always** use the `common::GetSlice` trait and the `.get_slice()` method to get a slice without causing the kernel to panic.  
  The problem with slicing in regular Rust, e.g. `foo[a..b]`, is that if someone tries to access with a range that is out of bounds of an array/string/slice, it will cause a panic at runtime, as a safety measure. Same thing when accessing an element.  
  Always use `foo.get(n)` instead of `foo[n]` and try to cover for the possibility of `Option::None`. Doing the regular way may work fine for applications, but never in the kernel. No possible panics should ever exist in kernel space, because then the whole OS would just stop working.

### <a name="testing-practices"> Testing Practices </a>

* It's always better to test boot (`make qemu` or `make virtualbox`) every time you make a change, because it is important to see how the OS boots and works after it compiles.  
  Even though Rust is a safety-oriented language, something as unstable as an in-dev operating system will have problems in many cases and may completely break on even the slightest critical change.  
  Also, make sure you check how the unmodified version runs on your machine before making any changes. Else, you won't have anything to compare to, and it will generally just lead to confusion. TLDR: Rebuild and test boot often.

* To run the ZFS tests:
 * Create the zfs.img only once. If one has not been created, run `make filesystem/apps/zfs/zfs.img` before booting into Redox.
 * Run `open zfs.img` to open the created ZFS image.
 * Run `file /home/LICENSE.md` twice to ensure ARC isn't broken.

## <a name="style-guidelines"> Style Guidelines </a>

### <a name="rust-style-guidelines"> Rust </a>

Since Rust is a relatively small and new language compared to others like C, there's really only one standard. Just follow the official Rust standards for formatting, and maybe run `rustfmt` on your changes, until we setup the CI system to do it automatically.

### <a name="git-style-guidelines"> Git </a>

* Commit messages should describe their changes in present tense, e.g. "`Add stuff to file.ext`" instead of "`added stuff to file.ext`".
* Try to remove useless duplicate/merge commits from PRs as these clutter up history, and may make it hard to read.
* Usually, when syncing your local copy with the master branch, you will want to rebase instead of merge. This is because it will create duplicate commits that don't actually do anything when merged into the master branch.
* When you start to make changes, you will want to create a separate branch, and keep the `master` branch of your fork identical to the main repository, so that you can compare your changes with the main branch and test out a more stable build if you need to.
* You should have a fork of the repository on GitHub and a local copy on your computer. The local copy should have two remotes; `upstream` and `origin`, `upstream` should be set to the main repository and `origin` should be your fork.

## <a name="other"> Other Ways to Contribute </a>

### <a name="graphic-design"> Graphic Design </a>

If you're a good designer, you can help with logos, UI design, app icons, other graphics (e.g. stock desktop backgrounds), etc. More information to come on this, for now just join [the chat](#chat) and ask about graphic design.
