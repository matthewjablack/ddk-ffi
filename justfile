# ====================
# React Native Bindings
# ====================

# Build the complete React Native bindings
uniffi:
  just uniffi-jsi
  just uniffi-turbo
  just build-ios
  just build-android
  echo ""
  echo "🎉 Uniffi build complete! 🎉"
  echo "🔥 Run 'just example-ios' to test the build"
  echo "⚠️ modify cpp/bennyblader-ddk-rn.cpp to #include 'ddk_ffi.hpp' ⚠️"

# Generate the JSI bindings
uniffi-jsi:
  cd {{justfile_directory()}}/ddk-ffi && uniffi-bindgen-react-native generate jsi bindings \
    --crate ddk_ffi --config ../ddk-rn/ubrn.config.toml \
    --ts-dir {{justfile_directory()}}/ddk-rn/src \
    --cpp-dir {{justfile_directory()}}/ddk-rn/cpp \
    {{justfile_directory()}}/ddk-ffi/src/ddk_ffi.udl

# Generate the TurboModule bindings
uniffi-turbo:
  cd {{justfile_directory()}}/ddk-rn && uniffi-bindgen-react-native generate jsi turbo-module ddk_ffi \
    --config ./ubrn.config.yaml \
    --native-bindings

# Build the iOS bindings
build-ios:
  cd {{justfile_directory()}}/ddk-rn && uniffi-bindgen-react-native build ios --and-generate

# Build the Android bindings
build-android:
  cd {{justfile_directory()}}/ddk-rn && uniffi-bindgen-react-native build android --and-generate 

# Build the example app with the React Native bindings
example:
  cd {{justfile_directory()}}/ddk-rn/example && pnpm install
  just example-ios
  just example-android

# Build the iOS example app
example-ios:
  cd {{justfile_directory()}}/ddk-rn/example/ios && RCT_NEW_ARCH_ENABLED=1 pod install && cd {{justfile_directory()}}/ddk-rn/example

# Build the Android example app
example-android:
  cd {{justfile_directory()}}/ddk-rn/example/android && ./gradlew build

# Clean all build artifacts and dependencies
clean:
  # Clean React Native bindings
  cd {{justfile_directory()}}/ddk-rn && rm -rf cpp/ddk_ffi.* cpp/ddk-rn.* cpp/UniffiCallInvoker.h src/ddk_ffi*.ts src/NativeDdkRn.ts ios/DdkRn.xcframework android/src/main/jniLibs lib ios/build android/build example/ios/build example/android/build example/android/app/build example/ios/Pods example/ios/Podfile.lock example/ios/DdkRnExample.xcworkspace src/index.tsx
  
  # Clean TypeScript/Node.js bindings
  cd {{justfile_directory()}}/ddk-ts && rm -rf node_modules dist target pnpm-lock.yaml
  cd {{justfile_directory()}}/ddk-ts/example && rm -rf node_modules dist

# Release the React Native bindings
rn-release:
  cd {{justfile_directory()}}/ddk-rn && node scripts/release.js

# Create binary archives for the React Native bindings
rn-release-archives:
  cd {{justfile_directory()}}/ddk-rn && node scripts/create-binary-archives.js

# ====================
# TypeScript (Node.js) Bindings
# ====================

# Build TypeScript bindings for current platform
ts-build:
    cd {{justfile_directory()}}/ddk-ts && yarn install && yarn build

# Build TypeScript bindings for all supported platforms (Darwin ARM64 and Linux x64)
ts-build-all:
    cd {{justfile_directory()}}/ddk-ts && yarn install && yarn build:darwin-arm64 && yarn build:linux-x64

# Run TypeScript example
ts-example:
    cd {{justfile_directory()}}/ddk-ts && yarn build
    cd {{justfile_directory()}}/ddk-ts/example && yarn install && yarn build && yarn start

# Run TypeScript tests
ts-test:
    cd {{justfile_directory()}}/ddk-ts && yarn test

# Release TypeScript package to npm
ts-release version:
    #!/usr/bin/env bash
    set -euo pipefail
    cd {{justfile_directory()}}/ddk-ts
    
    # Check if working directory is clean
    if [[ -n $(git status --porcelain) ]]; then
        echo "Error: Working directory is not clean. Please commit or stash changes."
        exit 1
    fi
    
    # Update version
    npm version {{version}} --no-git-tag-version
    
    # Build for all platforms
    echo "Building for all platforms..."
    # yarn build:darwin-arm64
    yarn build
    
    # Run tests
    echo "Running tests..."
    yarn test
    
    # Commit changes
    git add -A
    git commit -m "chore(ddk-ts): release v{{version}}"
    
    # Create tag
    git tag "v{{version}}"
    
    # Push to GitHub
    git push -u origin node-bindings
    git push origin "v{{version}}"
    
    # Publish to npm
    echo "Publishing to npm..."
    npm publish --access public
    
    echo "✅ Released ddk-ts v{{version}} successfully!"