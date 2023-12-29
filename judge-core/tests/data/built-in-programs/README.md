# judger-test-collection

## How to use test collection

As of now, all executables need to be built prior to testing.

use GNU make:

```bash
mkdir -p {build,dist}
cmake -B build --install-prefix $(pwd)/dist
cmake --build build --parallel
cmake --install build
```

or use Ninja:

```bash
mkdir -p {build,dist}
cmake -B build --install-prefix $(pwd)/dist -G Ninja .
cmake --build build
cmake --install build
```

Then you will get all test file on `dist/`

## Trouble Shooting

You might get errors like:
`g++: error: unrecognized command line option ‘-std=gnu++20’; did you mean ‘-std=gnu++2a’?`
While running the provided bash in this repository.

For resolution run the following commands if you are using Ubuntu in Github Codespaces:

``` sh
sudo add-apt-repository -y ppa:ubuntu-toolchain-r/test
sudo apt install -y g++-11
# Alternate the priority of g++ version usage
sudo update-alternatives --install /usr/bin/g++ g++ /usr/bin/g++-11 100
```
