# Called from native code (src/main/cpp/weft_jni.cpp and Hermes' PlatformUnicodeJava.cpp).
-keep class net.mioyi.weftui.Native { native <methods>; }
-keep interface net.mioyi.weftui.NativeHost { *; }
-keep class * implements net.mioyi.weftui.NativeHost { *; }
-keep class com.facebook.hermes.unicode.AndroidUnicodeUtils { public static *; }
