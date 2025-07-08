use std::env::args;

use pkg::{
    package::{Package, PackageError},
    PackageName,
};

use cookbook::WALK_DEPTH;

fn main() -> Result<(), PackageError> {
    let names: Vec<_> = args()
        .skip(1)
        .map(PackageName::new)
        .collect::<Result<_, _>>()?;

    let packages = Package::new_recursive(&names, WALK_DEPTH)?;
    for package in packages {
        println!("{}", package.name);
    }

    Ok(())
}
