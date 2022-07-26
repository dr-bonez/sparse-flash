# sparse-flash

## Usage

```
sudo $(which sparse-flash) --progress --input=/path/to/sparse/file.img /dev/sdX
```

## TAR Files

This tool only supports POSIX format TAR files. Using a GNU TAR file will result
in panics. To generate the correct kind of tar file, and write to stdout, you
can use the options for TAR in the below example:

```
tar --format=posix -cS -f- /path/to/sparse/file.img | sudo $(which sparse-flash) --progress --stdin-tar /dev/sdX
```
