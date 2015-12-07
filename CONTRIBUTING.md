# Contributing to Redox

<!-- TODO Write an introduction here -->

#### Index
- [Slack Chat](#slack)
- [GitHub Issues](#gh-issues)
- [Pull Requests](#prs)
- [Creating a Pull Request](#creating-a-pr)
- [Best Practices/Guidelines](#best-practices)

#### Other links
- [redox-os.org](http://redox-os.org)

<a name="slack"/>
## Slack Chat

The quickest and most open way to communicate with the Redox team is with Slack. Currently, the only way to join our slack team is by sending an email to [info@redox-os.org](mailto:info@redox-os.org), which might take a little while, since it's not automated. We're currently working on an easier way to do this, but this is the most convenient way right now.

<a name="gh-issues"/>
## GitHub Issues

A bit more formal way of communication with fellow Redox devs, but a little less quick and convienent like the Slack chat (unless of course you aren't in it yet, which if you're going to be involved in this project really at all, it is recommended that you request to join). These are for more specific topics, simply put, issues try to state something more than ask.

<a name="prs"/>
## Pull Requests

It's completely fine to just submit a small pull request without first making an issue or something, but if it's a big change that will require a lot of planning and reviewing, it's best you start with writing an issue first.

<a name="creating-a-pr"/>
## Creating a Pull Request

1. Fork the repository
2. Clone the original repository to your local PC using one of the following commands based on the protocol you are using:
    - HTTPS:`git clone https://github.com/redox-os/redox.git`
    - SSH:`git clone git@github.com:redox-os/redox.git --origin upstream --recursive`
    - Then rebase: `git rebase upstream master`
    Use HTTPS if you don't know which one to use. (Recommended: learn about SSH if you don't want to have to login every time you push/pull!)
3. Add your fork with
    - HTTPS:`git remote add origin https://github.com/your-username/redox.git`
    - SSH:`git remote add origin git@github.com:your-username/redox.git --origin upstream --recursive`
4. Alternatively, if you already have a fork and copy of the repo, you can simply check to make sure you're up-to-date
    - Fetch the upstream:`git fetch upstream master` 
    - Rebase with local commits:`git rebase upstream master`
    - Update the submodules:`git submodule update --init`
5. Optionally create a separate branch (recommended if you're making multiple changes simultaneously)
6. Make changes
7. Commit (`git add . --all; git commit -m "my commit"`)
8. Test your changes with `make qemu` or `make virtualbox` (you might have to use `make qemu_no_kvm`)
9. Pull from upstream (`git fetch upstream; git rebase upstream/master`) (Note: try not to use `git pull`, it is equivalent to doing `git fetch upstream; git merge`, which is not usually preferred for local repositories, although it is fine in some cases.)
10. Repeat step 7 to make sure the rebase still works
11. Push to your fork (`git push origin my-branch`)
12. Create a pull request
13. Describe your changes
14. Submit!

<a name="best-practices"/>
## Best Practices/Guidelines
<!-- TODO add this section to the index/TOC -->

#### General
<!-- TODO fill out this section -->

#### Testing Practices

- It's always better to test boot every time you make a change, because it is important to see how the OS boots and works after it compiles. Even though Rust is a safety-oriented language, something as unstable as an in-dev operating system will have problems in many cases and may completely break on even the slightest critical change. Also, make sure you check how the unmodified version runs on your machine before making any changes. Else, you won't have anything to compare to, and it will generally just lead to confusion. TLDR; Build/rebuild often.

- To run the ZFS tests:
    - Create the zfs.img only once. If one has not been created, run `make filesystem/apps/zfs/zfs.img` before booting into Redox.
    - Run `open zfs.img` to open the created ZFS image.
    - Run `file /home/LICENSE.md` twice to ensure ARC isn't broken.

#### Kernel

- When trying to access a slice, **always** use the `common::GetSlice` trait and use the `.get_slice()` method to get a slice without causing the kernel to panic. The problem with slicing in regular Rust, e.g. `foo[a..b]`, is that if someone tries to access with a range that is out of bounds of an array/string/slice, it will cause a panic at runtime, as a safety measure. Same thing when accessing an element. Always use `foo.get(n)` instead of `foo[n]` and try to cover for the possibility of `Option::None`. Doing the regular way may work fine for applications, but never in the kernel. No possible panics should ever exist in kernel space.

#### Style Guidelines
<!-- TODO fill out this section -->

#### Interactions with Other Projects
<!-- TODO fill out this section -->

#### Applications vs Kernel
<!-- TODO fill out this section -->

#### Low-Hanging Fruit - Easy Targets for Newbies
<!-- TODO improve this section -->

- If you're not fluent in Rust:
    - Documentation
    - Using/testing Redox, filing issues for bugs and needed features
    - Web dev (redox website, separate repo)
    - Unit tests
- If you are fluent in Rust, but not OS Development:
    - Apps
    - Shell (Ion) development
    - Package manager (Oxide) development
    - High-level code
