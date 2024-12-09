#!/bin/bash -xe
tar -xJf archive.tar.xz
ln -sf libmeetingsdk.so zoom-meeting-sdk-linux/libmeetingsdk.so.1
exit $?
