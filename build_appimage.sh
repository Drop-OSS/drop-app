#!/bin/sh

# run this from the root of the git repo to make this work

git_dir="./"
target_dir="$git_dir/src-tauri/target/"
appimage_dir="$git_dir/build/appimage"
appdir="$appimage_dir/drop-oss-app.d"

rm -f $appdir/usr/bin/* $appdir/usr/lib/*

# set up the repo
git submodule init
git submodule update --recursive

# set up yarn and build
yarn
yarn tauri build

# install binaries in the appdir, then the libraries
cp $target_dir/release/drop-app $appdir/usr/bin
for i in $(readelf -d "$target_dir/release/drop-app" |grep NEEDED |cut -d'[' -f2 |tr -d ]);
do
	sudo install -g $USER -o $USER -Dm755 "$(ls -L1 /usr/lib/$i)" $appdir/usr/lib
done

wget -O $appimage_dir/appimagetool https://github.com/AppImage/AppImageKit/releases/download/continuous/appimagetool-x86_64.AppImage

cd $appimage_dir
chmod u+x appimagetool
appimagetool $APPDIR
