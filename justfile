uniffi:
  just uniffi-jsi
  just uniffi-turbo
  just build-ios
  just build-android
  echo ""
  echo "🎉 Uniffi build complete! 🎉"
  echo "🔥 Run 'just example-ios' to test the build"
  echo "⚠️ modify cpp/bennyhodl-ddk-rn.cpp to #include 'ddk_ffi.hpp' ⚠️"

uniffi-jsi:
  cd {{justfile_directory()}}/ddk-ffi && uniffi-bindgen-react-native generate jsi bindings \
    --crate ddk_ffi --config ../ddk-rn/ubrn.config.toml \
    --ts-dir {{justfile_directory()}}/ddk-rn/src \
    --cpp-dir {{justfile_directory()}}/ddk-rn/cpp \
    {{justfile_directory()}}/ddk-ffi/src/ddk_ffi.udl

uniffi-turbo:
  cd {{justfile_directory()}}/ddk-rn && uniffi-bindgen-react-native generate jsi turbo-module ddk_ffi \
    --config ./ubrn.config.yaml \
    --native-bindings

build-ios:
  cd {{justfile_directory()}}/ddk-rn && uniffi-bindgen-react-native build ios --and-generate

build-android:
  cd {{justfile_directory()}}/ddk-rn && uniffi-bindgen-react-native build android --and-generate 

example:
  cd {{justfile_directory()}}/ddk-rn/example && pnpm install
  just example-ios
  just example-android

example-ios:
  cd {{justfile_directory()}}/ddk-rn/example/ios && RCT_NEW_ARCH_ENABLED=1 pod install && cd {{justfile_directory()}}/ddk-rn/example

example-android:
  cd {{justfile_directory()}}/ddk-rn/example/android && ./gradlew build

clean:
  cd {{justfile_directory()}}/ddk-rn && rm -rf cpp/ddk_ffi.* cpp/ddk-rn.* cpp/UniffiCallInvoker.h src/ddk_ffi*.ts src/NativeDdkRn.ts ios/DdkRn.xcframework android/src/main/jniLibs lib ios/build android/build example/ios/build example/android/build example/android/app/build example/ios/Pods example/ios/Podfile.lock example/ios/DdkRnExample.xcworkspace src/index.tsx