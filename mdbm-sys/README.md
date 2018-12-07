# Building

On OSX, you can build mdbm from scratch. In another directory:

```bash
brew install coreutils cppunit readline openssl
git clone https://github.com/yahoo/mdbm
cd mdbm
make
```

Then move to this directory, and build with:

```bash
C_INCLUDE_PATH="$MDBMDIR/include" DYLD_LIBRARY_PATH="$MDBMDIR/src/lib/object" LIBRARY_PATH="$MDBMDIR/src/lib/object" cargo build
```