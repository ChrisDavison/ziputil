# ziputil

[![Rust](https://github.com/ChrisDavison/ziputil/actions/workflows/rust.yml/badge.svg)](https://github.com/ChrisDavison/ziputil/actions/workflows/rust.yml)

This is a little utility to help you get files out of a `zip` without having to extract every file.

Currently, there are three commands:

- list -- print list of files in the zip
- choose -- list files in the zip and extract a choice
- view -- list files in the zip and then `cat` (basically only useful for plaintext, where _plaintext notes backups_ was my primary usecase)

![](screenshots/ziputil.png)
