#!/bin/sh

# Generic script to create a package with Project Builder in mind
# There should only be one version of this script for all projects!

FRAMEWORK="$1"
VARIANT="$2"

PACKAGE="$FRAMEWORK"
PACKAGE_RESOURCES="pkg-support/resources"

echo "Building package for $FRAMEWORK.framework"
echo "Will fetch resources from $PACKAGE_RESOURCES"
echo "Will create the package $PACKAGE.pkg"

# create a copy of the framework
mkdir -p build/pkg-tmp
/Developer/Tools/CpMac -r "build/$FRAMEWORK.framework" build/pkg-tmp/

./package build/pkg-tmp "pkg-support/$PACKAGE.info" -d  build -r "$PACKAGE_RESOURCES" 

# remove temporary files
rm -rf build/pkg-tmp

# compress
(cd build; tar -zcvf "$PACKAGE.pkg.tar.gz" "$PACKAGE.pkg")

