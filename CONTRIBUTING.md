# Contributing to Redox

**Thank you for your interest in contributing to Redox!**

This document will outline the basics of where to start if you wish to contribute to the project. There are many ways to help us out and and we appreciate all of them. We look forward to **your contribution!**

## Code Of Conduct

We follow the [Rust Code Of Conduct](https://www.rust-lang.org/policies/code-of-conduct).

## License

In general, your contributions to Redox are governed by the [MIT License](https://en.wikipedia.org/wiki/MIT_License). Each project repository has a `LICENSE` file that provides the license terms for that project.

Please review the LICENSE for the project you are contributing to.

On [this](https://doc.redox-os.org/book/ch01-02-philosophy.html) page we explain why we use the MIT license.

## Chat

Join us on [Matrix Chat](https://doc.redox-os.org/book/ch13-01-chat.html) to discuss issues or ask questions.

## Suggestions for Contributions

(Before starting to contribute you **must** read the [FAQ](https://www.redox-os.org/faq/) and the [Redox Book](https://doc.redox-os.org/book/))

You can contribute to the Redox documentation and code on these repositories:

(The order is based on difficulty, easy things first)

- [Website](https://gitlab.redox-os.org/redox-os/website)
- [Book](https://gitlab.redox-os.org/redox-os/book)
- [Build System Configuration](https://gitlab.redox-os.org/redox-os/redox) - Our main repository
- [Cookbook](https://gitlab.redox-os.org/redox-os/cookbook) - Package system
- [relibc](https://gitlab.redox-os.org/redox-os/relibc) - Redox C Library
- [Drivers](https://gitlab.redox-os.org/redox-os/drivers)
- [Kernel](https://gitlab.redox-os.org/redox-os/kernel)

### Places to Contribute

If you aren't fluent in Rust:

- Write documentation
- Use and test Redox, fill issues for bugs or needed features (verify the repository issues before)
- Web development on the [website](https://gitlab.redox-os.org/redox-os/website)
- Write unit tests (may require minimal knowledge of Rust)

If you are fluent in Rust, but not operating system Development:

- Port programs written in Rust to Redox (in most cases you need to port crates, be aware of missing functions on relibc, porting without these functions will make patches dirty)
- [relibc](https://gitlab.redox-os.org/redox-os/relibc) - Redox C Library
- The [Ion Shell](https://gitlab.redox-os.org/redox-os/ion)
- [Package Manager](https://gitlab.redox-os.org/redox-os/pkgutils)

If you are fluent in Rust, and have experience with operating system development:

- Familiarize yourself with the repository and codebase
- Grep for `TODO`, `FIXME`, `BUG`, `UNOPTIMIZED`, `REWRITEME`, `DOCME`, and `PRETTYFYME` and fix the code you find
- Update older code to remove warnings
- Improve and optimize code, especially in the kernel
- Write drivers

For those who want to contribute to the Redox GUI, our GUI strategy has recently changed.

- We are porting the [COSMIC compositor](https://github.com/pop-os/cosmic-comp), help wanted.
- Redox is in the process of adopting other Rust-lang GUIs such as [Iced](https://iced.rs) and [Slint](https://slint-ui.com/). Please check out those projects if this is your area of interest.
- OrbTk is in maintenance mode, and its developers have moved to other projects such as the ones below. There is currently no Redox-specific GUI development underway.

## Tracking Issues Index

We use the Tracking Issues Index to ease the development workflow, you can find them on [this](https://gitlab.redox-os.org/redox-os/redox/-/issues/1384) page.

## Build System

You can find the Redox build system organization and commands on [this](https://doc.redox-os.org/book/ch08-06-build-system-reference.html) page.

## Developer FAQ

You can see the most common questions and problems on [this](https://doc.redox-os.org/book/ch09-07-developer-faq.html) page.

## Porting Software

You can read how to use the Cookbook recipe system to port applications on [this](https://doc.redox-os.org/book/ch09-03-porting-applications.html) page.

## Libraries and APIs

You can read [this](https://doc.redox-os.org/book/ch09-06-libraries-apis.html) page to learn about the libraries and APIs used in Redox.

## Development Tips

You can find important tips on [this](https://doc.redox-os.org/book/ch09-02-coding-and-building.html#development-tips) section.

## References

We maintain a list of wikis, articles and videos to learn Rust, OS development and computer science on [this](https://doc.redox-os.org/book/ch09-08-references.html) page.

If you are skilled there's a possibility that they could improve your knowledge in some way.

## Best Practices and Guidelines

You can read the best practices and guidelines on [this](https://doc.redox-os.org/book/ch11-00-best-practices.html) chapter.

## Style Guidelines

### Rust

Since **Rust** is a relatively small and new language compared to others like C and C++, there's really only one standard. Just follow the official Rust standards for formatting, and maybe run `rustfmt` on your changes, until we setup the CI system to do it automatically.

### Git

Please follow our [Git style](https://doc.redox-os.org/book/ch12-04-creating-proper-pull-requests.html) for pull requests.

## GitLab

### Issues

To know how to create issues on the Redox GitLab, read [this](https://doc.redox-os.org/book/ch12-05-filing-issues.html) page.

### Pull Requests

Please follow [our process](https://doc.redox-os.org/book/ch12-04-creating-proper-pull-requests.html) for creating proper pull requests.

## Other Ways to Contribute

If you aren't good on coding, but you still want to help keep the project going, you can contribute and support in a variety of ways! We'll try to find a way to use anything you have to offer. 

### Design

If you're a good designer, whether it's 2D graphics, 3D graphics, interfaces, web design, you can help. We need logos, UI design, UI skins, app icons, desktop backgrounds, etc.

- [Redox backgrounds](https://gitlab.redox-os.org/redox-os/backgrounds) - You can send your wallpapers on this repository.
- [Redox assets](https://gitlab.redox-os.org/redox-os/assets) - You can send your logos, icons and themes on this repository.

If you have questions about the graphic design, ask us on the [Chat](https://doc.redox-os.org/book/ch13-01-chat.html).

### Donate to Redox

If you are interested in donating to the Redox OS Nonprofit, you can find instructions [here](https://www.redox-os.org/donate/).
