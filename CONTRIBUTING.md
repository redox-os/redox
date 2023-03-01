# Contributing to Redox

_**Thank you for your interest in contributing to Redox!** This document will outline the basics of where to start if you wish to contribute to the project. There are many ways to help us out and and we appreciate all of them. We look forward to **your contribution**!_

## Code Of Conduct

We follow the [Rust Code Of Conduct](https://www.rust-lang.org/policies/code-of-conduct).

## License

All contributions are under the MIT license.

- [Our Philosophy](https://doc.redox-os.org/book/ch01-02-philosophy.html)

## Low-Hanging Fruit - Easy Targets for Newbies

If you're not fluent in Rust:

 * Writing documentation
 * Using/testing Redox, filing issues for bugs and needed features
 * Web development ([Redox website, separate repo](https://gitlab.redox-os.org/redox-os/website))
 * Writing unit tests (may require minimal knowledge of rust)

If you are fluent in Rust, but not OS Development:

 * Port applications written in Rust to Redox
 * Rewritten-in-Rust libc ([relibc](https://gitlab.redox-os.org/redox-os/relibc))
 * Shell ([Ion](https://gitlab.redox-os.org/redox-os/ion))
 * Package manager ([pkgutils](https://gitlab.redox-os.org/redox-os/pkgutils))

If you are fluent in Rust, and have experience with OS Dev:

 * Familiarize yourself with the repository and codebase
 * Grep for `TODO`, `FIXME`, `BUG`, `UNOPTIMIZED`, `REWRITEME`, `DOCME`, and `PRETTYFYME` and fix the code you find.
 * Update older code to remove warnings.
 * Improve and optimize code, especially in the kernel

For those who want to contribute to the Redox GUI, our GUI strategy has recently changed.

 * OrbTk is now sunsetting, and its developers have moved to other projects such as the ones below. There is currently no Redox-specific GUI development underway.
 * Redox is in the process of adopting other Rust-lang GUIs such as [Iced](https://iced.rs) and [Slint](https://slint-ui.com/). Please check out those projects if this is your area of interest.

## Best Practices and Guidelines

- [Redox Book Guide](https://doc.redox-os.org/book/ch11-00-best-practices.html)

### Testing Practices

- [Redox Book Guide](https://doc.redox-os.org/book/ch09-03-testing-practices.html)

## Style Guidelines

### Rust

Since **Rust** is a relatively small and new language compared to others like _C_, there's really only one standard. Just follow the official Rust standards for formatting, and maybe run `rustfmt` on your changes, until we setup the CI system to do it automatically.

### Git

Please follow our [Git style for pull requests](https://doc.redox-os.org/book/ch12-04-creating-proper-pull-requests.html).

## GitLab

### Issues

You will need to have a Redox GitLab account to file an issue, and registration can take a few days.

- [Redox Book Guide](https://doc.redox-os.org/book/ch13-03-gitlab-issues.html)

### Pull Requests

Please follow our process for [creating proper pull requests](https://doc.redox-os.org/book/ch12-04-creating-proper-pull-requests.html).


## External Links

* [redox-os.org](https://redox-os.org)
* [rust-os-comparison](https://github.com/flosse/rust-os-comparison)
* [rust-lang.org](http://rust-lang.org)

## Chat

Join us on [Matrix Chat](https://doc.redox-os.org/book/ch13-01-chat.html) to discuss issues or ask questions.

## Other Ways to Contribute

If you're not big on coding, but you still want to help keep the project going, you can still contribute/support in a variety of ways! We'll try to find a way to use anything you have to offer. 

### Book ToDos

- [Book README](https://gitlab.redox-os.org/redox-os/book/-/blob/master/README.md)

### Design

If you're a good designer, whether it's _2D graphics, 3D graphics, interfaces, web design, you can help. We need logos, UI design, UI skins, app icons, desktop backgrounds, etc_. More information to come on this in the future, for now just join the [Chat](https://doc.redox-os.org/book/ch13-01-chat.html) and ask about graphic design.

### Donate to Redox

If you are interested in donating to the Redox OS nonprofit, you can find instructions [here](https://www.redox-os.org/donate/).
