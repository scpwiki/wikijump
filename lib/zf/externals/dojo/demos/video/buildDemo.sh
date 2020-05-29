#!/bin/sh

cd ../../util/buildscripts
./build_mini.sh action=release version=1.2.0-alpha profileFile=../../demos/uploader/demo.profile.js 
cd ../../release/dojo/
# create an archive file to send somewhere
tar -czvf ../uploaderDemo.tar.gz . --exclude dijit --exclude dojox --exclude util
