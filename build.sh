ARCH=$(uname -m)

# build tauri apps
# NO_STRIP=true yarn tauri build -- --verbose

# unpack appimage
APPIMAGE=$(ls ./src-tauri/target/release/bundle/appimage/*.AppImage)
"$APPIMAGE" --appimage-extract

# strip binary
APPIMAGE_UNPACK="./squashfs-root"
find $APPIMAGE_UNPACK -name "*.so" -exec strip {} \;

APPIMAGETOOL=$(echo "appimagetool-$ARCH.AppImage")
wget -O $APPIMAGETOOL "https://github.com/AppImage/appimagetool/releases/download/1.9.0/$APPIMAGETOOL"
chmod +x $APPIMAGETOOL

./$APPIMAGETOOL $APPIMAGE_UNPACK
