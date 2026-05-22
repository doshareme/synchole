//! Kotlin/Java binding strategy.
//!
//! Production builds should generate an Android AAR that exposes a small Kotlin
//! API over JNI and keeps long-running sync work on a Rust-owned Tokio runtime.

use jni::objects::JClass;
use jni::sys::jint;
use jni::JNIEnv;

#[allow(non_snake_case)]
pub extern "system" fn Java_dev_synchole_Synchole_nativeVersionMajor(
    _env: JNIEnv<'_>,
    _class: JClass<'_>,
) -> jint {
    0
}
