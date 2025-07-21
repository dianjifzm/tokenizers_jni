extern crate jni;
extern crate tokenizers;

use jni::{JNIEnv};
use jni::objects::{JClass, JObject, JValue, JString};
use jni::sys::{jlong, jobject};
use tokenizers::Tokenizer;

// Constants
const NATIVE_ALLOCATION_FAILED_EXCEPTION: &str = "co/huggingface/tokenizers/exceptions/NativeAllocationFailedException";


#[no_mangle]
pub extern "system" fn Java_co_huggingface_tokenizers_tokenizer_Tokenizer_from_file(mut _env: JNIEnv, _class: JClass, file_path: JString) -> jobject {
    // 获取文件路径字符串
    let file_path: String = match _env.get_string(&file_path) {
        Ok(path) => path.into(),
        Err(_) => {
            let _ = _env.throw_new(NATIVE_ALLOCATION_FAILED_EXCEPTION, "Couldn't get file path");
            return JObject::null().as_raw();
        }
    };

    // 从文件加载 tokenizer
    let tokenizer: Result<Box<Tokenizer>, String> = match Tokenizer::from_file(&file_path) {
        Ok(tokenizer) => Ok(Box::new(tokenizer)),
        Err(e) => Err(format!("Failed to load tokenizer from file: {:?}", e))
    };

    // 检查是否加载成功
    if tokenizer.is_err() {
        let _ = _env.throw_new(NATIVE_ALLOCATION_FAILED_EXCEPTION, "Unable to load Tokenizer from file");
        return JObject::null().as_raw();
    }

    // 将 tokenizer 转换为原始指针并存储为 handle
    let handle = Box::into_raw(tokenizer.unwrap()) as jlong;
    
    // 创建 Java Tokenizer 对象
    match _env.new_object("co/huggingface/tokenizers/tokenizer/Tokenizer", "()V", &[]) {
        Ok(j_tokenizer) => {
            // 设置 handle 字段
            let _ = _env.set_field(&j_tokenizer, "handle", "J", JValue::Long(handle));
            return j_tokenizer.as_raw();
        }
        Err(_) => {
            // 如果创建 Java 对象失败，需要释放已分配的内存
            unsafe {
                let _ = Box::from_raw(handle as *mut Tokenizer);
            }
            let _ = _env.throw_new(NATIVE_ALLOCATION_FAILED_EXCEPTION, "Unable to create Tokenizer object");
            return JObject::null().as_raw();
        }
    }
}