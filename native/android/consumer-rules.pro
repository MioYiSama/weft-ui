# Called from native code (src/main/cpp/weft_jni.cpp and Hermes' PlatformUnicodeJava.cpp).
-keep class dev.weft.Native { native <methods>; }
-keep interface dev.weft.NativeHost { *; }
-keep class * implements dev.weft.NativeHost { *; }
-keep class com.facebook.hermes.unicode.AndroidUnicodeUtils { public static *; }
