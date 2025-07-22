extern crate jni;
extern crate tokenizers;

use jni::{JNIEnv};
use jni::objects::{JClass, JObject, JValue, JString};
use jni::sys::{jlong, jobject, jintArray};
use tokenizers::Tokenizer;

// Constants
const NATIVE_ALLOCATION_FAILED_EXCEPTION: &str = "co/huggingface/tokenizers/exceptions/NativeAllocationFailedException";


#[no_mangle]
pub extern "system" fn Java_co_huggingface_tokenizers_Tokenizer_fromFile(mut _env: JNIEnv, _class: JClass, file_path: JString) -> jobject {
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
    match _env.new_object("co/huggingface/tokenizers/Tokenizer", "()V", &[]) {
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

// Tokenizer encode 方法
#[no_mangle]
pub unsafe extern "system" fn Java_co_huggingface_tokenizers_Tokenizer_encode(
    mut _env: JNIEnv, 
    _obj: JObject, 
    text: JString
) -> jobject {
    // 获取 tokenizer 指针
    let handle = match _env.get_field(&_obj, "handle", "J") {
        Ok(val) => val.j().unwrap(),
        Err(_) => {
            let _ = _env.throw_new(NATIVE_ALLOCATION_FAILED_EXCEPTION, "Failed to get tokenizer handle");
            return JObject::null().as_raw();
        }
    };
    
    let tokenizer = &*(handle as *mut Tokenizer);
    
    // 获取输入文本
    let input_text: String = match _env.get_string(&text) {
        Ok(s) => s.into(),
        Err(_) => {
            let _ = _env.throw_new(NATIVE_ALLOCATION_FAILED_EXCEPTION, "Failed to get input text");
            return JObject::null().as_raw();
        }
    };
    
    // 执行编码
    let encoding = match tokenizer.encode(input_text.as_str(), false) {
        Ok(enc) => enc,
        Err(_) => {
            let _ = _env.throw_new(NATIVE_ALLOCATION_FAILED_EXCEPTION, "Failed to encode text");
            return JObject::null().as_raw();
        }
    };
    
    // 获取 token IDs
    let ids = encoding.get_ids();
    let tokens = encoding.get_tokens();
    let offsets = encoding.get_offsets();
    
    // 创建 Java int 数组存储 token IDs
    let ids_array = match _env.new_int_array(ids.len() as i32) {
        Ok(arr) => arr,
        Err(_) => {
            let _ = _env.throw_new(NATIVE_ALLOCATION_FAILED_EXCEPTION, "Failed to create ids array");
            return JObject::null().as_raw();
        }
    };
    
    let ids_i32: Vec<i32> = ids.iter().map(|&x| x as i32).collect();
    let _ = _env.set_int_array_region(&ids_array, 0, &ids_i32);
    
    // 创建 Java String 数组存储 tokens
    let string_class = match _env.find_class("java/lang/String") {
        Ok(cls) => cls,
        Err(_) => {
            let _ = _env.throw_new(NATIVE_ALLOCATION_FAILED_EXCEPTION, "Failed to find String class");
            return JObject::null().as_raw();
        }
    };
    
    let tokens_array = match _env.new_object_array(tokens.len() as i32, &string_class, JObject::null()) {
        Ok(arr) => arr,
        Err(_) => {
            let _ = _env.throw_new(NATIVE_ALLOCATION_FAILED_EXCEPTION, "Failed to create tokens array");
            return JObject::null().as_raw();
        }
    };
    
    for (i, token) in tokens.iter().enumerate() {
        let java_string = match _env.new_string(token) {
            Ok(s) => s,
            Err(_) => continue,
        };
        let _ = _env.set_object_array_element(&tokens_array, i as i32, &java_string);
    }
    
    // 创建 Offset 数组
    let offset_class = match _env.find_class("co/huggingface/tokenizers/Offset") {
        Ok(cls) => cls,
        Err(_) => {
            let _ = _env.throw_new(NATIVE_ALLOCATION_FAILED_EXCEPTION, "Failed to find Offset class");
            return JObject::null().as_raw();
        }
    };
    
    let offsets_array = match _env.new_object_array(offsets.len() as i32, &offset_class, JObject::null()) {
        Ok(arr) => arr,
        Err(_) => {
            let _ = _env.throw_new(NATIVE_ALLOCATION_FAILED_EXCEPTION, "Failed to create offsets array");
            return JObject::null().as_raw();
        }
    };
    
    for (i, offset) in offsets.iter().enumerate() {
        let offset_obj = match _env.new_object("co/huggingface/tokenizers/Offset", "(II)V", &[
            JValue::Int(offset.0 as i32),
            JValue::Int(offset.1 as i32),
        ]) {
            Ok(obj) => obj,
            Err(_) => {
                let _ = _env.throw_new(NATIVE_ALLOCATION_FAILED_EXCEPTION, "Failed to create Offset object");
                return JObject::null().as_raw();
            }
        };
        let _ = _env.set_object_array_element(&offsets_array, i as i32, &offset_obj);
    }
    
    // 创建 Encoding 对象
    match _env.new_object("co/huggingface/tokenizers/Encoding", "([I[Ljava/lang/String;[Lco/huggingface/tokenizers/Offset;)V", &[
        JValue::Object(&JObject::from(ids_array)),
        JValue::Object(&JObject::from(tokens_array)),
        JValue::Object(&JObject::from(offsets_array)),
    ]) {
        Ok(encoding_obj) => encoding_obj.as_raw(),
        Err(_) => {
            let _ = _env.throw_new(NATIVE_ALLOCATION_FAILED_EXCEPTION, "Failed to create Encoding object");
            JObject::null().as_raw()
        }
    }
}

// Tokenizer decode 方法
#[no_mangle]
pub unsafe extern "system" fn Java_co_huggingface_tokenizers_Tokenizer_decode(
    mut _env: JNIEnv, 
    _obj: JObject, 
    ids: jintArray
) -> jobject {
    // 获取 tokenizer 指针
    let handle = match _env.get_field(&_obj, "handle", "J") {
        Ok(val) => val.j().unwrap(),
        Err(_) => {
            let _ = _env.throw_new(NATIVE_ALLOCATION_FAILED_EXCEPTION, "Failed to get tokenizer handle");
            return JObject::null().as_raw();
        }
    };
    
    let tokenizer = &*(handle as *mut Tokenizer);
    
    // 获取 token IDs
    let ids_array = unsafe { jni::objects::JIntArray::from_raw(ids) };
    let ids_len = match _env.get_array_length(&ids_array) {
        Ok(len) => len,
        Err(_) => {
            let _ = _env.throw_new(NATIVE_ALLOCATION_FAILED_EXCEPTION, "Failed to get array length");
            return JObject::null().as_raw();
        }
    };
    
    let mut ids_vec = vec![0i32; ids_len as usize];
    let _ = _env.get_int_array_region(&ids_array, 0, &mut ids_vec);
    
    let ids_u32: Vec<u32> = ids_vec.iter().map(|&x| x as u32).collect();
    
    // 执行解码
    let decoded_text = match tokenizer.decode(&ids_u32, false) {
        Ok(text) => text,
        Err(_) => {
            let _ = _env.throw_new(NATIVE_ALLOCATION_FAILED_EXCEPTION, "Failed to decode tokens");
            return JObject::null().as_raw();
        }
    };
    
    // 返回解码后的文本
    match _env.new_string(&decoded_text) {
        Ok(jstr) => jstr.as_raw(),
        Err(_) => {
            let _ = _env.throw_new(NATIVE_ALLOCATION_FAILED_EXCEPTION, "Failed to create result string");
            JObject::null().as_raw()
        }
    }
}