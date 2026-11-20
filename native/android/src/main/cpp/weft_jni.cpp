// JNI glue of the Android shell: the kernel's C ABI (weft/runtime.h) as the
// external functions of net.mioyi.weftui.Native, and the kernel's host callbacks
// as calls to a net.mioyi.weftui.NativeHost. Everything runs on the main thread,
// which the JVM has attached; strings cross as UTF-8 byte arrays because JNI's
// modified UTF-8 cannot carry every string a bundle may produce.

#include <jni.h>

#include <fbjni/fbjni.h>

#include <cstdint>
#include <cstring>
#include <memory>
#include <string>
#include <vector>

#include "weft/runtime.h"

namespace {

JavaVM *g_vm = nullptr;

/// One runtime with the shell object its callbacks go to.
struct Session {
  jobject host = nullptr;
  jmethodID commit = nullptr;
  jmethodID schedule = nullptr;
  jmethodID supports = nullptr;
  jmethodID call = nullptr;
  jmethodID log = nullptr;
  /// The bytecode, aligned and alive for the runtime's lifetime as the kernel requires.
  std::unique_ptr<uint8_t[]> bytecode;
  weft_runtime *runtime = nullptr;
};

JNIEnv *env() {
  JNIEnv *env = nullptr;
  g_vm->GetEnv(reinterpret_cast<void **>(&env), JNI_VERSION_1_6);
  return env;
}

/// Reports and clears an exception thrown by a callback into the shell: the
/// kernel's caller is native code that cannot propagate it.
void clear(JNIEnv *env) {
  if (env->ExceptionCheck()) {
    env->ExceptionDescribe();
    env->ExceptionClear();
  }
}

jbyteArray bytes(JNIEnv *env, const void *data, size_t len) {
  jbyteArray array = env->NewByteArray(static_cast<jsize>(len));
  if (array != nullptr && len > 0) {
    env->SetByteArrayRegion(array, 0, static_cast<jsize>(len), static_cast<const jbyte *>(data));
  }
  return array;
}

jbyteArray bytes(JNIEnv *env, const char *text) {
  return bytes(env, text, std::char_traits<char>::length(text));
}

std::vector<uint8_t> copy(JNIEnv *env, jbyteArray array) {
  std::vector<uint8_t> out;
  if (array != nullptr) {
    out.resize(static_cast<size_t>(env->GetArrayLength(array)));
    env->GetByteArrayRegion(array, 0, static_cast<jsize>(out.size()), reinterpret_cast<jbyte *>(out.data()));
  }
  return out;
}

Session *from(void *ctx) { return static_cast<Session *>(ctx); }

Session *from(jlong handle) { return reinterpret_cast<Session *>(handle); }

void commit(void *ctx, const uint8_t *ops, size_t len) {
  JNIEnv *e = env();
  // The batch is only valid during this call: the buffer is a view, not a copy.
  jobject buffer = e->NewDirectByteBuffer(const_cast<uint8_t *>(ops), static_cast<jlong>(len));
  e->CallVoidMethod(from(ctx)->host, from(ctx)->commit, buffer);
  clear(e);
  e->DeleteLocalRef(buffer);
}

void schedule(void *ctx, uint64_t delay_ms) {
  JNIEnv *e = env();
  e->CallVoidMethod(from(ctx)->host, from(ctx)->schedule, static_cast<jlong>(delay_ms));
  clear(e);
}

bool supports(void *ctx, const char *module) {
  JNIEnv *e = env();
  jbyteArray name = bytes(e, module);
  bool result = e->CallBooleanMethod(from(ctx)->host, from(ctx)->supports, name);
  if (e->ExceptionCheck()) {
    clear(e);
    result = false;
  }
  e->DeleteLocalRef(name);
  return result;
}

void call(void *ctx, uint32_t id, const char *module, const char *method, const uint8_t *args,
          size_t len) {
  JNIEnv *e = env();
  jbyteArray m = bytes(e, module);
  jbyteArray f = bytes(e, method);
  jbyteArray a = bytes(e, args, len);
  e->CallVoidMethod(from(ctx)->host, from(ctx)->call, static_cast<jint>(id), m, f, a);
  clear(e);
  e->DeleteLocalRef(m);
  e->DeleteLocalRef(f);
  e->DeleteLocalRef(a);
}

void log(void *ctx, int level, const char *msg, size_t len) {
  JNIEnv *e = env();
  jbyteArray text = bytes(e, msg, len);
  e->CallVoidMethod(from(ctx)->host, from(ctx)->log, static_cast<jint>(level), text);
  clear(e);
  e->DeleteLocalRef(text);
}

}  // namespace

extern "C" {

JNIEXPORT jint JNI_OnLoad(JavaVM *vm, void *) {
  g_vm = vm;
  // Hermes attaches its finalizer thread through fbjni.
  return facebook::jni::initialize(vm, [] {});
}

JNIEXPORT jlong JNICALL Java_net_mioyi_weftui_Native_create(JNIEnv *e, jobject, jobject host,
                                                            jbyteArray bytecode,
                                                            jlong layout_hash, jstring version,
                                                            jstring environment) {
  auto session = std::make_unique<Session>();
  jclass type = e->GetObjectClass(host);
  session->commit = e->GetMethodID(type, "commit", "(Ljava/nio/ByteBuffer;)V");
  session->schedule = e->GetMethodID(type, "schedule", "(J)V");
  session->supports = e->GetMethodID(type, "supports", "([B)Z");
  session->call = e->GetMethodID(type, "call", "(I[B[B[B)V");
  session->log = e->GetMethodID(type, "log", "(I[B)V");
  e->DeleteLocalRef(type);
  if (e->ExceptionCheck()) return 0;  // NoSuchMethodError propagates to the caller.
  session->host = e->NewGlobalRef(host);

  const auto len = static_cast<size_t>(e->GetArrayLength(bytecode));
  session->bytecode.reset(new uint8_t[len]);
  e->GetByteArrayRegion(bytecode, 0, static_cast<jsize>(len),
                        reinterpret_cast<jbyte *>(session->bytecode.get()));

  const char *chars = e->GetStringUTFChars(version, nullptr);
  const std::string os = chars != nullptr ? chars : "";
  if (chars != nullptr) e->ReleaseStringUTFChars(version, chars);

  // The environment JSON has ASCII keys and values (numbers and fixed strings).
  const char *json = e->GetStringUTFChars(environment, nullptr);
  const weft_host callbacks{
      .ctx = session.get(),
      .commit = commit,
      .schedule = schedule,
      .supports = supports,
      .call = call,
      .log = log,
      .version = os.c_str(),
      .environment = json,
  };
  if (json != nullptr) {
    session->runtime = weft_runtime_new(&callbacks, session->bytecode.get(), len,
                                        static_cast<uint64_t>(layout_hash));
    e->ReleaseStringUTFChars(environment, json);
  }
  if (session->runtime == nullptr) {
    e->DeleteGlobalRef(session->host);
    return 0;
  }
  return reinterpret_cast<jlong>(session.release());
}

JNIEXPORT void JNICALL Java_net_mioyi_weftui_Native_environment(JNIEnv *e, jobject, jlong handle, jstring json) {
  const char *bytes = e->GetStringUTFChars(json, nullptr);
  if (bytes == nullptr) return;
  weft_runtime_environment(from(handle)->runtime, bytes, std::strlen(bytes));
  e->ReleaseStringUTFChars(json, bytes);
}

JNIEXPORT void JNICALL Java_net_mioyi_weftui_Native_tick(JNIEnv *, jobject, jlong handle) {
  weft_runtime_tick(from(handle)->runtime);
}

JNIEXPORT jboolean JNICALL Java_net_mioyi_weftui_Native_dispatch(JNIEnv *e, jobject, jlong handle,
                                                                 jint scope, jint slot,
                                                                 jbyteArray payload) {
  const std::vector<uint8_t> bytes = copy(e, payload);
  return weft_runtime_dispatch(from(handle)->runtime, static_cast<uint32_t>(scope),
                               static_cast<uint32_t>(slot), bytes.data(), bytes.size());
}

JNIEXPORT void JNICALL Java_net_mioyi_weftui_Native_resolve(JNIEnv *e, jobject, jlong handle,
                                                            jint id, jboolean ok,
                                                            jbyteArray result) {
  const std::vector<uint8_t> bytes = copy(e, result);
  weft_runtime_resolve(from(handle)->runtime, static_cast<uint32_t>(id), ok, bytes.data(),
                       bytes.size());
}

JNIEXPORT void JNICALL Java_net_mioyi_weftui_Native_free(JNIEnv *e, jobject, jlong handle) {
  Session *session = from(handle);
  if (session == nullptr) return;
  weft_runtime_free(session->runtime);
  e->DeleteGlobalRef(session->host);
  delete session;
}

}  // extern "C"
