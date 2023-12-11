# Cookbook

Package system of Redox.

This repository contains the system source code and packages inside the `recipes` folder.

- A recipe can be a software port or system package (they use `pkgar` or `tar.gz` formats).

**Read [this](https://doc.redox-os.org/book/ch09-03-porting-applications.html) page before porting programs to Redox**

In order for this repository to be useful, it must be set up with an environment
from the [redox](https://gitlab.redox-os.org/redox-os/redox) repository.

[![MIT licensed](https://img.shields.io/badge/license-MIT-blue.svg)](./LICENSE)

- [Recipe Categories](#recipe-categories)
- [Package Policy](#package-policy)
    - [Cross-Compilation](#cross-compilation)
    - [Library Linking](#library-linking)
    - [ABI stability](#abi-stability)
    - [Checksum](#checksum)
    - [License](#license)
- [Testing Area](#testing-area)
    - [Suggestions for TODOs](#suggestions-for-todos)
- [Repository Layout](#repository-layout)
- [TODO](#todo)

### Recipe Categories

The categories inside the `recipes` folder.

- `backends` - Middlewares, like SDL2
- `core` - System components (included on `server` build)
- `demos` - Software with demos
- `development` - Any software used for development, like compilers and dependency managers
- `documentation` - Software used for documentation
- `emulators` - Console emulators or compatibility/translation layers
- `examples` - Softwares with examples
- `games` - Any kind of game
- `gui` - Graphical interfaces
- `icons` - Icon packs
- `libraries` - Software with functions for other softwares, like OpenSSL
- `network` - Networking tools
- `other` - Software that can't fit on other categories
- `shells` - Terminal interpreters
- `sound` - Software used for sound processing/production
- `tests` - Software used to test other softwares
- `toolkits` - Software used to create other softwares with a framework-like approach
- `tools` - Text editors, terminal tools and any other kind of tools
- `tui` - Graphical terminal interfaces
- `video` - Video players and video processing/production tools
- `wip` - Software that needs porting or incomplete recipes

### Package Policy

Before sending your recipe to upstream (to become a public package), you must follow these rules:

#### Cross-Compilation

- All recipes must use our cross-compilers, a Cookbook [template](https://doc.redox-os.org/book/ch09-03-porting-applications.html#templates) does this automatically but it's not always possible, study the build system of your program/library to find these options or patch the configuration files.
- Don't hardcode the CPU architecture on the recipe script.

#### Library Linking

- Keep the static linking of libraries, there's an exception if the package become bigger than 100MB, big libraries/runtimes like LLVM can be dynamically linked.

#### ABI stability

- Respect the ABI separation of the libraries, for example, if `openssl1` is available and some program need `openssl3`, you will create a recipe for `openssl3` and not rename the `openssl1`, as it will break the dependent packages.

#### Checksum

- If your recipe download a tarball, you will need to create a BLAKE3 hash for it. You can learn how to do it [here](https://doc.redox-os.org/book/ch09-03-porting-applications.html#create-a-blake3-hash-for-your-recipee).

#### License

- Don't package programs/libraries lacking a license.
- Verify if the program has some license violation, in case of doubt ask us on the [chat](https://doc.redox-os.org/book/ch13-01-chat.html).

### Testing Area

Work-in-progress software ports goes to the `wip` category, be aware of these items during your packaging process:

- A recipe is considered ready if it's mostly working inside of Redox.
- All WIP recipes must have a `#TODO` on the beginning of the `recipe.toml` and explain what is missing.
- BLAKE3 hashes for tarballs are optional (quick testing workflow)
- Try to keep the recipe with the latest stable version of the program (the porting process can take months).
- Once the recipe is ready, add the BLAKE3 hash if needed and move the folder to the appropriate category.

#### Suggestions for TODOs

These TODOs improve the packagers cooperation and understanding.

- `not compiled or tested` - It means that your recipe is fully configured and don't lack necessary dependencies.
- `missing script for x, see insert-the-link-for-build-instructions-here` - It means that your recipe is lacking the cross-compilation script for some build system, where `x` is the build system name. After `see` you will insert the link for the build instructions of the program or library, it will help other packagers to insert the script for you.
- `missing dependencies, see insert-the-link-for-required-dependencies-here` - It means that the `dependencies = []` section is incomplete.
- `probably wrong script, see insert-the-link-for-build-instructions-here` - It means that you don't know if your script will work.
- `probably wrong template, see insert-the-link-for-build-instructions-here` - It means that you don't know if the Cookbook template will work.
- `probably missing dependencies, see insert-the-link-for-required-dependencies-here` - It means that you don't know if the required dependencies are satisfied.

Other TODOs are specific and won't be covered on this list.

### Repository Layout

- `.cargo` - Cargo configuration.
- `bin` - LLVM and pkg-config CPU targets.
- `recipes` - Package configuration files.
- `src` - Package system source code.

### TODO

- Convert old recipes to TOML, see [this](https://gitlab.redox-os.org/redox-os/cookbook/-/issues/174) tracking issue.
- Remove the scripts after full TOML conversion.
