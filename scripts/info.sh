#!/bin/bash

stable_user_id=""
stable_version=""

app_support="$HOME/Library/Application Support"
has_app_data() {
    [ -n "$(ls -A "$1" 2>/dev/null)" ]
}

# Mirrors resolve_app_folder in crates/storage/src/global.rs.
stable_app_folder="$app_support/anarlog"
if has_app_data "$app_support/hyprnote" && ! has_app_data "$app_support/anarlog"; then
    stable_app_folder="$app_support/hyprnote"
fi

if [ -f "$stable_app_folder/store.json" ]; then
    stable_user_id=$(jq -r '."auth-user-id" // empty' "$stable_app_folder/store.json")
fi

if [ -d "/Applications/Anarlog.app" ]; then
    stable_version=$(defaults read /Applications/Anarlog.app/Contents/Info.plist CFBundleShortVersionString 2>/dev/null || echo "")
elif [ -d "/Applications/Char.app" ]; then
    stable_version=$(defaults read /Applications/Char.app/Contents/Info.plist CFBundleShortVersionString 2>/dev/null || echo "")
elif [ -d "/Applications/Hyprnote.app" ]; then
    stable_version=$(defaults read /Applications/Hyprnote.app/Contents/Info.plist CFBundleShortVersionString 2>/dev/null || echo "")
fi

cat << EOF
{
    "stable": {
        "userId": "${stable_user_id}",
        "version": "${stable_version}"
    }
}
EOF
