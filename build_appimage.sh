#!/bin/sh

# run this from the root of the git repo to make this work

arch="$(uname -m)"
git_dir="$PWD"
target_dir="$git_dir/src-tauri/target/"
appimage_dir="$git_dir/build/appimage"
appdir="$appimage_dir/drop-app.d"

build() {
	# set up the repo
	git submodule init
	git submodule update --recursive

	# set up yarn and build
	yarn
	yarn tauri build
}

rm -f $appdir/usr/bin/* $appdir/usr/lib/*

if [[ ! "$1" == "--nobuild" ]]; then
	build	
fi

# install binaries in the appdir, then the libraries
cp $target_dir/release/drop-app $appdir/usr/bin
for i in $(readelf -d "$target_dir/release/drop-app" |grep NEEDED |cut -d'[' -f2 |tr -d ]);
do
	install -g 1000 -o 1000 -Dm755 "$(ls -L1 /usR/LIB/$i)" $appdir/usr/lib
done

wget -O $appimage_dir/appimagetool https://github.com/AppImage/AppImageKit/releases/download/continuous/appimagetool-$arch.AppImage

cd $appimage_dir
chmod u+x appimagetool
appimagetool $appdir
