# Cookbook

A collection of software ports for Redox.

### Categories

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

In order for this repository to be useful, it must be set up with an environment
from the [redox](https://gitlab.redox-os.org/redox-os/redox) repository.

### Package Policy

When you send your recipe to upstream (to become a public package), you must follow these rules:

- Keep the static linking of libraries, there's an exception if the library/runtime is bigger than 50MB, big libraries/runtimes like LLVM can be dynamically linked.
- Respect the ABI separation of the packages, for example, if `openssl1` is available and some program need `openssl3`, you will create a recipe for `openssl3` and not rename the `openssl1`, as it will break the ABI of the dependent packages.
- If your recipe download a tarball you need to create a BLAKE3 hash for it, you can learn how to do it [here](https://doc.redox-os.org/book/ch09-03-porting-applications.html#create-a-blake3-hash-for-your-recipee).
- Verify if the recipe has some license violation, in case of doubt ask us on the [chat](https://doc.redox-os.org/book/ch13-01-chat.html).
- If your recipe is incomplete you will add it on the `wip` folder, you don't need to insert a BLAKE3 hash (it's quicker to test new tarball versions without checksum) but you need to insert a `#TODO` on the beginning of the `recipe.toml` and explain what's missing. Once the recipe is ready, add the BLAKE3 hash if needed and move the folder to the appropriate category.

[![MIT licensed](https://img.shields.io/badge/license-MIT-blue.svg)](./LICENSE)