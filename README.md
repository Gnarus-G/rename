# rename

A cli tool to rename files in bulk.

## Why?

I don't like the perl-rename or the rename tool in linux. I could learn to get better at them but I don't want to. Anyway regex can get pretty gaudy and it's not intuitive to me to come with the right pattern on the fly in a timely manner. So I'm experimenting here with a simpler syntax: `hello(n:int)->hi(n)` which reads better and looks self-explanatory. But I don't rename files that often so maybe it doesn't matter.

## Install (Unix)

```sh
export VER=$(wget -qO- https://github.com/Gnarus-G/rename/releases/latest | grep -oP 'v\d+\.\d+\.\d+' | tail -n 1);
curl -L https://github.com/Gnarus-G/rename/releases/download/$VER/rn-$OSTYPE.tar.gz -o rename.tar.gz
tar -xzvf rename.tar.gz rn
# Allow to able to run it from anywhere [Optional]
sudo mv rn /usr/local/bin
```

## Usage

Grab a binary (linux or mac) in [releases](https://github.com/Gnarus-G/rename/releases) or run the install script above.

```sh
./rn simple --help
```

For example to replace `file1`, or `file99` to `1renamed.txt` or `99renamed.txt`

### Experimental MRP (Match Replace Protocol)

```sh
./rn simple "file(n:int)->(n)renamed.txt" file*
```

### Regular Expression

```sh
./rn regex "file(\d+)" '${1}renamed.txt' file*
```

## Demo

![simplescreenrecorder-2023-01-01_23 51 24](https://user-images.githubusercontent.com/37311893/210196100-96190c6e-9597-4755-a0a0-de86ca407d4a.gif)

## Note

The "simple" match and replace syntax is still in development. The happy path pretty works, and the parser gives comprehensive error messages.
