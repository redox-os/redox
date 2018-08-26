# Contributing to Redox

_**Thank you for your interest in contributing to Redox!** This document will outline the basics of where to start if you wish to contribute to the project. There are many ways to help us out and and we appreciate all of them. We look forward to **your contribution**!_

## Index

* [Communication](#communication)
 * [Chat](#chat)
 * [GitLab Issues](#issues)
 * [Pull Requests](#prs)
 * [Discourse](#discourse)
 * [Reddit](#reddit)
 * [News](#news)
* [Code Contributions](#code-contributions)
 * [Low-Hanging Fruit - Easy Targets for Newbies](#easy-targets)
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
 * [Patreon](#patreon)

## <a name="extern-links"> Other External Links </a>

* [redox-os.org](https://redox-os.org)
* [rust-os-comparison](https://github.com/flosse/rust-os-comparison)
* [rust-lang.org](http://rust-lang.org)

## <a name="communication"> Communication </a>

### <a name="chat"> Chat </a>

The quickest and most open way to **communicate with the Redox team** is on our **chat server**. Currently, you can only get an invite by sending an email request to [info@redox-os.org](mailto:info@redox-os.org), which might take a little while, since it's not automated. Simply say you'd like to join the chat. We're working on an better way to do this, but this is the best way right now.

### <a name="issues"> GitLab Issues </a>

A bit more formal way of communication with fellow Redox devs, but a little less quick and convenient like the chat. Submit an issue when you run into problems compiling, testing, or just would like to discuss a certain topic, be it _features, code style, code inconsistencies, minor changes and fixes, etc._

### <a name="prs"> Pull Requests </a>

It's fine to just submit a small pull request without first making an issue or asking in the chat, **unless** it's a significant change that will require a lot of planning and reviewing. Also see [Creating a Pull Request](#creating-a-pr) and [Git Style Guidelines](#git-style-guidelines).

### <a name="discourse"> Discourse </a>

We have a **discourse forum** at [discourse.redox-os.org](https://discourse.redox-os.org). This is the best way to discuss more general topics that aren't about specific things that need to be addressed one way or another. You can sign up like any other website.

### <a name="reddit"> Reddit </a>

You can also find **Redox on Reddit** in [/r/rust/](https://www.reddit.com/r/rust) and [/r/redox/](https://www.reddit.com/r/redox). Redox news and discussion is posted on the latter, and Rust news and discussion, as well as some Redox posts, is on the former.

### <a name="news"> News </a>

News and updates for Redox are posted at [redox-os.org/news](https://redox-os.org/news). It's more one-way than the other things on this list, but it should provide a good summary of what's been going on with the project lately. It's usually updated weekly, but with some exceptions. A mailing list may be included eventually, but it's not set up right now.

## <a name="code-contributions"> Code Contributions </a>

### <a name="easy-targets"> Low-Hanging Fruit - Easy Targets for Newbies </a>

#### If you're not fluent in Rust:

 * Writing _documentation_
 * **Using/testing Redox**, filing issues for bugs and needed features
 * **Web development** ([Redox website, separate repo](https://gitlab.redox-os.org/redox-os/website))
 * **Writing unit tests** (may require minimal knowledge of rust)

#### If you are fluent in Rust, but not OS Development:

 * **Apps** development
 * **Shell** ([Ion](https://gitlab.redox-os.org/redox-os/ion)) development
 * **Package management** ([pkgutils](https://gitlab.redox-os.org/redox-os/pkgutils)) development
 * Other high-level code tasks

#### If you are fluent in Rust, and have experience with OS Dev:

 * Familiarize yourself with the repository and codebase
 * Grep for `TODO`, `FIXME`, `BUG`, `UNOPTIMIZED`, `REWRITEME`, `DOCME`, and `PRETTYFYME` and fix the code you find.
 * **Improve and optimize code, especially in the kernel**

### <a name="creating-a-pr"> Creating a Pull Request </a>

**1**. _**Fork**_ the repository

**2**. Clone the _original repository_ to your local PC using one of the following commands based on the protocol you are using:
 * HTTPS:`git clone https://gitlab.redox-os.org/redox-os/redox.git --origin upstream --recursive`
 * SSH:`git clone git@gitlab.redox-os.org:redox-os/redox.git --origin upstream --recursive`
 * Then rebase: `git rebase upstream master`
 If you use HTTPS, you will have to log in each time when pushing to your fork. (Recommended: learn about git SSH support, it logs in automatically using SSH keys)
 
**3**. **Add** your fork with
 * HTTPS:`git remote add origin https://gitlab.redox-os.org/your-username/redox.git`
 * SSH:`git remote add origin git@gitlab.redox-os.org:your-username/redox.git`
 
**4**. Alternatively, if you already have a fork and copy of the repo, you can simply check to **make sure you're up-to-date**
 * Pull the upstream:`git pull upstream --rebase`
 * Update the submodules:`git submodule update --recursive --init`
 
**5**. Create a _**separate branch**_ (recommended if you're making multiple changes simultaneously) (`git checkout -b my-branch`)

**6**. _Make changes_

**7**. **Commit** (`git add <item(s) you changed>; git commit`) and write your commit message

**8**. Optionally run [rustfmt](https://github.com/rust-lang-nursery/rustfmt) on the _files you changed_ and commit again if it did anything (check with `git diff` first)

**9**. Test your changes by **cleaning** (`make clean; git clean -Xfd`) and **building** with `make qemu` (you might have to use `make qemu kvm=no`) or `make virtualbox`.
(see [Best Practices and Guidelines](#best-practices))

**10**. _**Pull**_ from upstream (`git pull upstream --rebase`) (Note: Make sure to include `--rebase`, as it will apply your changes on top of the changes you just pulled, allowing for a much cleaner merge)

**11**. Repeat step **9** to make sure the rebase still builds and starts

**12**. Push to **your fork** (`git push origin <branch>`), `<branch>` being the branch you created earlier

**13**. Create a _pull request_

**14**. If your changes are _minor_, you can just describe them in a paragraph or less. If they're _major_, please fill out the provided form.

**15. Submit!**

## <a name="best-practices"> Best Practices and Guidelines </a>

### <a name="general"> General </a>

* **Remember to do a `git rebase -i upstream/master` before you send your patch!**
* **Make sure your code is readable, commented, and well-documented.**
* **Don't hesitate to ask for help, comments or suggestions!**
* **Before implementing something, discuss it! Open an issue, or ask in the chat.**

##### On the more technical side:
* Test, test, and test!
* Follow the style conventions (See [rust style guidelines](#rust-style-guidelines))
* Use `std::mem::replace` and `std::mem::swap` when you can.
* `libredox` should be 1-to-1 with the official `libstd`.
* Prefer `.into()` and `.to_owned()` over `.to_string()`.
* Prefer passing references to the data over owned data. (Don't take `String`, take `&str`. Don't take `Vec<T>` take `&[T]`).
* Use generics, traits, and other abstractions Rust provides.
* Avoid using lossy conversions (for example: don't do `my_u32 as u16 == my_u16`, prefer `my_u32 == my_u16 as u32`).
* Prefer in place (`box` keyword) when doing heap allocations.
* Prefer platform independently sized integer over pointer sized integer (`u32` over `usize`, for example).
* Follow the usual idioms of programming, such as "composition over inheritance", "let your program be divided in smaller pieces", and "resource acquisition is initialization".
* When `unsafe` is unnecessary, don't use it. **Longer safe code is better than shorter unsafe code!**
* Be sure to mark parts that need work with `TODO`, `FIXME`, `BUG`, `UNOPTIMIZED`, `REWRITEME`, `DOCME`, and `PRETTYFYME`. Always elaborate on these messages, too. Nothing is more annoying than seeing a `TODO` and not knowing how to actually fix it.
* Use the compiler hint attributes, such as `#[inline]`, `#[cold]`, etc. when it makes sense to do so.
* Check the [chat](#chat), [the website](http://redox-os.org/news), and [the Rust subreddit](https://www.reddit.com/r/rust) frequently.

### <a name="kernel"> Kernel </a>

* When trying to access a slice, **always** use the `common::GetSlice` trait and the `.get_slice()` method to get a slice without causing the kernel to panic.
  The problem with slicing in regular Rust, e.g. `foo[a..b]`, is that if someone tries to access with a range that is out of bounds of an array/string/slice, it will cause a panic at runtime, as a safety measure. Same thing when accessing an element.
  Always use `foo.get(n)` instead of `foo[n]` and try to cover for the possibility of `Option::None`. Doing the regular way may work fine for applications, but never in the kernel. No possible panics should ever exist in kernel space, because then the whole OS would just stop working.

### <a name="testing-practices"> Testing Practices </a>

* It's always better to test boot (`make qemu` or `make virtualbox`) every time you make a change, because it is important to see how the OS boots and works after it compiles.
  Even though Rust is a safety-oriented language, something as unstable and low-level as an in-dev operating system will almost certainly have problems in many cases and may completely break on even the slightest critical change.
  Also, make sure you check how the unmodified version runs on your machine before making any changes. Else, you won't have anything to compare to, and it will generally just lead to confusion. TLDR: Rebuild and test boot often.

* To run the **ZFS** tests:
 * Create the zfs.img only once. If one has not been created, run `make filesystem/apps/zfs/zfs.img` before booting into Redox.
 * Run `open zfs.img` to open the created ZFS image.
 * Run `file /home/LICENSE.md` twice to ensure ARC isn't broken.

## <a name="style-guidelines"> Style Guidelines </a>

### <a name="rust-style-guidelines"> Rust </a>

Since **Rust** is a relatively small and new language compared to others like _C_, there's really only one standard. Just follow the official Rust standards for formatting, and maybe run `rustfmt` on your changes, until we setup the CI system to do it automatically.

### <a name="git-style-guidelines"> Git </a>

* You should have a fork of the repository on **GitHub** and a local copy on your computer. The local copy should have two remotes; `upstream` and `origin`, `upstream` should be set to the main repository and `origin` should be your fork.
* When you start to make changes, you will want to create a separate branch, and keep the `master` branch of your fork identical to the main repository, so that you can compare your changes with the main branch and test out a more stable build if you need to.
* Usually, when syncing your local copy with the master branch, you'll want to rebase instead of merge. This is because it will create duplicate commits that don't actually do anything when merged into the master branch. You can do this in one command with `git pull upstream --rebase`. This will pull from the upstream, then roll back to the current state of the upstream, and "replay" your changes on top of it. Make sure you commit before doing this, though. Git won't be able to rebase if you don't.
* Prefer to omit the `-m` when using `git commit`. This opens your editor and should help get you in the habit of writing longer commit messages.
* Commit messages should describe their changes in present tense, e.g. "`Add stuff to file.ext`" instead of "`added stuff to file.ext`". This makes sense as sometimes when you revert back, then run through commits one-by-one, you want to see what a commit will do, instead of just what the person did when they made the commit. It's also just being consistent.
* Try to remove useless duplicate/merge commits from PRs as these don't do anything except clutter up history and make it harder to read.

## <a name="other"> Other Ways to Contribute </a>

If you're not big on coding, but you still want to help keep the project going, you can still contribute/support in a variety of ways! We'll try to find a way to use anything you have to offer. 

### <a name="design"> Design </a>

If you're a good designer, whether it's _2D graphics, 3D graphics, interfaces, web design, you can help. We need logos, UI design, UI skins, app icons, desktop backgrounds, etc_. More information to come on this in the future, for now just join [the chat](#chat) and ask about graphic design.

### <a name="patreon"> Patreon </a>

Our **BDFL**, [jackpot51](https://gitlab.redox-os.org/jackpot51), has a [Patreon campaign](https://www.patreon.com/redox_os)! **All money received will go towards Redox OS development**. If you donate, you will be listed in the **Redox credits** as one of the people that made Redox OS possible. You'll also get other rewards the more you donate. However, please don't donate if you can't afford it.

<!--

POSSIBLE OTHER TOPICS TO INCLUDE
- Merch (maybe in the future)
- Sound Design (things like notifications, popups, etc. for orbital)
- Video Production/Motion Graphics (tutorials, introduction videos, etc.)
- Non-Rust programming, scripting, etc. (if we even have a need for this)
- Hosting/download/git mirrors (this is not needed right now, but when downloads increase it may be necessary)

-->

