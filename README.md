# rename

A cli tool to rename files in bulk.

## Install (Unix)

```sh
export VER=$(wget -qO- https://github.com/Gnarus-G/rename/releases/latest | grep -oP 'v\d+\.\d+\.\d+' | tail -n 1);
curl -L https://github.com/Gnarus-G/rename/releases/download/$VER/rn-$OSTYPE.tar.gz -o rename.tar.gz
tar -xzvf rename.tar.gz rn
# Allow to able to run it from anywhere [Optional]
sudo mv rn /usr/local/bin
```

## Usage

Grab a binary (linux or mac) in [releases](https://github.com/Gnarus-G/rename/releases)

```sh
./rn simple --help
```
