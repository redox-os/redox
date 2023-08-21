# Contributing to Redox

_**Thank you for your interest in contributing to Redox!** This document will outline the basics of where to start if you wish to contribute to the project. There are many ways to help us out and and we appreciate all of them. We look forward to **your contribution**!_

## Code Of Conduct

We follow the [Rust Code Of Conduct](https://www.rust-lang.org/policies/code-of-conduct).

## License

In general, your contributions to Redox are governed by the MIT License. Each project repository has a LICENSE file that provides the license terms for that project.

Please review the LICENSE for the project you are contributing to.

- [Our Philosophy](https://doc.redox-os.org/book/ch01-02-philosophy.html)

## Chat

Join us on [Matrix Chat](https://doc.redox-os.org/book/ch13-01-chat.html) to discuss issues or ask questions.

## Suggestions for Contributions

(Before start to contribute you **must** read the [FAQ](https://www.redox-os.org/faq/) and the [Book](https://doc.redox-os.org/book/))

You can contribute to Redox documentation/code on these repositories:

(The order is based on difficulty, easy things first)

- [Website](https://gitlab.redox-os.org/redox-os/website)
- [Book](https://gitlab.redox-os.org/redox-os/book)
- [Main Repository (build system/config)](https://gitlab.redox-os.org/redox-os/redox)
- [Cookbook (all system components/ported software)](https://gitlab.redox-os.org/redox-os/cookbook)
- [Redox C Library](https://gitlab.redox-os.org/redox-os/relibc)
- [Drivers](https://gitlab.redox-os.org/redox-os/drivers)
- [Kernel](https://gitlab.redox-os.org/redox-os/kernel)

### Important Places to Contribute

If you're not fluent in Rust:

 - Write documentation
 - Use/test Redox, filing issues for bugs and needed features (verify the GitLab issues before)
 - Web development ([Redox website](https://gitlab.redox-os.org/redox-os/website))
 - Write unit tests (may require minimal knowledge of Rust)

If you are fluent in Rust, but not OS Development:

 - Port applications written in Rust to Redox (missing support on relibc will make patches dirty)
 - Rewritten-in-Rust libc ([relibc](https://gitlab.redox-os.org/redox-os/relibc))
 - Shell ([Ion](https://gitlab.redox-os.org/redox-os/ion))
 - Package Manager ([pkgutils](https://gitlab.redox-os.org/redox-os/pkgutils))

If you are fluent in Rust, and have experience with OS Dev:

 - Familiarize yourself with the repository and codebase
 - Grep for `TODO`, `FIXME`, `BUG`, `UNOPTIMIZED`, `REWRITEME`, `DOCME`, and `PRETTYFYME` and fix the code you find
 - Update older code to remove warnings
 - Improve and optimize code, especially in the kernel
 - Write drivers

For those who want to contribute to the Redox GUI, our GUI strategy has recently changed.

 - OrbTk is now sunsetting, and its developers have moved to other projects such as the ones below. There is currently no Redox-specific GUI development underway.
 - Redox is in the process of adopting other Rust-lang GUIs such as [Iced](https://iced.rs) and [Slint](https://slint-ui.com/). Please check out those projects if this is your area of interest.

## Tracking Issues

We use tracking issues to ease the development workflow, you can find them on this page:

- [Tracking issues index](https://gitlab.redox-os.org/redox-os/redox/-/issues/1384)

## Build System

You can find the Redox build system organization and commands on this page:

- [Build System Quick Reference](https://doc.redox-os.org/book/ch08-06-build-system-reference.html)

## Porting Software

You can read how to use the Cookbook recipe system to port applications on this page:

- [Porting Applications Using Recipes](https://doc.redox-os.org/book/ch09-03-porting-applications.html)

## Developer FAQ

You can see the most common questions and problems on this page:

- [Develper FAQ](https://doc.redox-os.org/book/ch09-05-developer-faq.html)

## Best Practices and Guidelines

- [Best Practices](https://doc.redox-os.org/book/ch11-00-best-practices.html)

## Style Guidelines

### Rust

Since **Rust** is a relatively small and new language compared to others like _C_, there's really only one standard. Just follow the official Rust standards for formatting, and maybe run `rustfmt` on your changes, until we setup the CI system to do it automatically.

### Git

Please follow our [Git style for pull requests](https://doc.redox-os.org/book/ch12-04-creating-proper-pull-requests.html).

## GitLab

### Issues

You will need to have a Redox GitLab account to file an issue, and registration can take a few days.

- [Redox Book Guide](https://doc.redox-os.org/book/ch12-05-issues.html)

### Pull Requests

Please follow our process for [creating proper pull requests](https://doc.redox-os.org/book/ch12-04-creating-proper-pull-requests.html).


## External Links

- [redox-os.org](https://redox-os.org)
- [rust-os-comparison](https://github.com/flosse/rust-os-comparison)
- [rust-lang.org](http://rust-lang.org)

## Other Ways to Contribute

If you're not big on coding, but you still want to help keep the project going, you can still contribute/support in a variety of ways! We'll try to find a way to use anything you have to offer. 

### Design

If you're a good designer, whether it's _2D graphics, 3D graphics, interfaces, web design, you can help. We need logos, UI design, UI skins, app icons, desktop backgrounds, etc_. More information to come on this in the future, for now just join the [Chat](https://doc.redox-os.org/book/ch13-01-chat.html) and ask about graphic design.

### Donate to Redox

If you are interested in donating to the Redox OS nonprofit, you can find instructions [here](https://www.redox-os.org/donate/).
