#!/bin/bash -xe
tar -xJf archive.tar.xz
ln -s libmeetingsdk.so zoom-meeting-sdk-linux/libmeetingsdk.so.1 -f
exit $?
