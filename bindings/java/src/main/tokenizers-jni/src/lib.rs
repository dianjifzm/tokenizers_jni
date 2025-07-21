mod glue;
mod helpers;

extern crate jni;
extern crate tokenizers;

use jni::{JNIEnv};
use jni::objects::{JClass, JObject, JValue, JString};
use jni::sys::{jint, jlong, jobject, jstring};

use tokenizers::models::bpe::BPE;
use tokenizers::tokenizer::Token;
use tokenizers::pre_tokenizers::whitespace::Whitespace;
use tokenizers::pre_tokenizers::byte_level::ByteLevel;

use helpers::string_vector_to_arraylist;
use glue::reinterpret_cast;

// Constants
const NATIVE_ALLOCATION_FAILED_EXCEPTION: &str = "co/huggingface/tokenizers/exceptions/NativeAllocationFailedException";
const STRING_DECODING_EXCEPTION: &str = "co/huggingface/tokenizers/exceptions/StringDecodingException";


// Pretokenizer

//// Whitespace
#[no_mangle]
pub extern "system" fn Java_co_huggingface_tokenizers_pretokenizers_WhitespacePretokenizer_allocate(_env: JNIEnv, _class: JClass, _obj: JObject) -> jlong {
    return Box::into_raw(Box::new(Whitespace)) as jlong;
}

#[no_mangle]
pub unsafe extern "system" fn Java_co_huggingface_tokenizers_pretokenizers_WhitespacePretokenizer_finalize(mut _env: JNIEnv, _obj: JObject) {
    match _env.get_field(&_obj, "handle", "J") {
        Ok(ptr) => {
            let _ = _env.set_field(&_obj, "handle", "J", JValue::Long(-1));
            let _boxed = Box::from_raw(ptr.j().unwrap() as *mut Whitespace);
        },
        Err(_) => { let _ = _env.throw_new(NATIVE_ALLOCATION_FAILED_EXCEPTION, "Unable to retrieve Whitespace ptr"); }
    };
}

#[no_mangle]
pub unsafe extern "system" fn Java_co_huggingface_tokenizers_pretokenizers_WhitespacePretokenizer_pretokenize(mut _env: JNIEnv, _obj: JObject, s: JString) -> jobject {
    // Retrieve Whitespace instance ptr and reinterpret_cast<Whitespace>
    let _whitespace = match _env.get_field(&_obj, "handle", "J"){
        Ok(ptr) => match ptr.j(){
            Ok(ptr) => Some(&mut *(ptr as *mut Whitespace)),
            Err(_) => {
                let _ = _env.throw_new(NATIVE_ALLOCATION_FAILED_EXCEPTION, "Failed to reinterpret Whitespace ptr");
                None
            }
        },
        Err(_) => {
            let _ = _env.throw_new(NATIVE_ALLOCATION_FAILED_EXCEPTION, "Failed to retrieve Whitespace ptr");
            None
        }
    };

    // Simple implementation - just return the input as a single token
    let input_str = _env.get_string(&s).unwrap().to_str().unwrap().to_string();
    let tokens = vec![input_str];
    match string_vector_to_arraylist(&mut _env, &tokens){
        Ok(jarray_tokens) => return jarray_tokens,
        _ => {
            let _ = _env.throw_new(NATIVE_ALLOCATION_FAILED_EXCEPTION, "");
            return JObject::null().as_raw();
        }
    }
}

//// Byte Level
#[no_mangle]
pub extern "system" fn Java_co_huggingface_tokenizers_pretokenizers_ByteLevelPretokenizer_allocate(_env: JNIEnv, _class: JClass, _obj: JObject) -> jlong {
    return Box::into_raw(Box::new(ByteLevel::default())) as jlong;
}

#[no_mangle]
pub unsafe extern "system" fn Java_co_huggingface_tokenizers_pretokenizers_ByteLevelPretokenizer_finalize(mut _env: JNIEnv, _obj: JObject) {
    // Finalize is very special call, let's handle it with extra care to be sure memory (if any) is desallocated properly
    match _env.get_field(&_obj, "handle", "J") {
        Ok(ptr) => {
            let _ = _env.set_field(&_obj, "handle", "J", JValue::Long(-1));
            let pretokenizer = reinterpret_cast::<ByteLevel>(ptr.j().unwrap());
            let _boxed = Box::from_raw(pretokenizer);
        },
        Err(_) => { let _ = _env.throw_new(NATIVE_ALLOCATION_FAILED_EXCEPTION, "Unable to retrieve ByteLevel ptr"); }
    };
}

#[no_mangle]
pub unsafe extern "system" fn Java_co_huggingface_tokenizers_pretokenizers_ByteLevelPretokenizer_pretokenize(mut _env: JNIEnv, _obj: JObject, s: JString) -> jobject {
    // Retrieve Whitespace instance ptr and reinterpret_cast<Whitespace>
    let _pretokenizer = reinterpret_cast::<ByteLevel>(_env.get_field(&_obj, "handle", "J").unwrap().j().unwrap());

    // Simple implementation - just return the input as a single token
    let input_str = _env.get_string(&s).unwrap().to_str().unwrap().to_string();
    let tokens = vec![input_str];
    match string_vector_to_arraylist(&mut _env, &tokens){
        Ok(jarray_tokens) => return jarray_tokens,
        _ => {
            let _ = _env.throw_new(NATIVE_ALLOCATION_FAILED_EXCEPTION, "");
            return JObject::null().as_raw();
        }
    }
}

#[no_mangle]
pub unsafe extern "system" fn Java_co_huggingface_tokenizers_pretokenizers_ByteLevelPretokenizer_decode(mut _env: JNIEnv, _obj: JObject, _words: JObject) -> jstring {
    // Retrieve ByteLevel instance ptr and reinterpret_cast<ByteLevel>
    let _pretokenizer = reinterpret_cast::<ByteLevel>(_env.get_field(&_obj, "handle", "J").unwrap().j().unwrap());

    // Simple implementation - just return empty string for now
    match _env.new_string(""){
        Ok(jstr) => return jstr.as_raw(),
        _ => {
            let _ = _env.throw_new(STRING_DECODING_EXCEPTION, "");
            return JObject::null().as_raw();
        }
    }
}

// BPE
#[no_mangle]
pub extern "system" fn Java_co_huggingface_tokenizers_models_BytePairEncoder_fromFiles(mut _env: JNIEnv, _class: JClass, vocabs: JString, merges: JString) -> jobject {
    let vocabs: String = _env.get_string(&vocabs)
        .expect("Couldn't get vocab file path")
        .into();

    let merges: String = _env.get_string(&merges)
        .expect("Couldn't get merges file path")
        .into();

    let bpe: Result<Box<BPE>, String> = match tokenizers::models::bpe::BPE::from_file(&vocabs, &merges).build() {
        Ok(bpe) => Ok(Box::new(bpe)),
        Err(e) => Err(format!("Failed to build BPE: {:?}", e))
    };

    if bpe.is_err() {
        let _ = _env.throw_new(NATIVE_ALLOCATION_FAILED_EXCEPTION, "Unable to allocate BytePairEncoder");
        return JObject::null().as_raw()
    }

    let handle = Box::into_raw(bpe.unwrap()) as jlong;
    match _env.new_object("Lco/huggingface/tokenizers/models/BytePairEncoder;", "()V", &[]){
        Ok(j_bpe) => {
            let _ = _env.set_field(&j_bpe, "handle", "J", JValue::Long(handle));
            return j_bpe.as_raw();
        }
        Err(_) => {
            let _ = _env.throw_new(NATIVE_ALLOCATION_FAILED_EXCEPTION, "Unable to set BytePairEncoder.handle");
            return JObject::null().as_raw();
        }
    }
}

#[no_mangle]
pub unsafe extern "system" fn Java_co_huggingface_tokenizers_models_BytePairEncoder_tokenize(mut _env: JNIEnv, _obj: JObject, _words: JObject) -> jobject {
    // Retrieve BytePairEncoder object
    let _bpe = reinterpret_cast::<BPE>(_env.get_field(&_obj, "handle", "J").unwrap().j().unwrap());

    // Simple implementation - just return empty token list for now
    let tokens: Vec<Token> = Vec::new();

    match _env.new_object("java/util/ArrayList", "(I)V", &[JValue::Int(tokens.len() as jint)]) {
        Ok(jarray_) => {
            return jarray_.as_raw()
        },
        Err(_) => {
            let _ = _env.throw_new(NATIVE_ALLOCATION_FAILED_EXCEPTION, "Failed to allocate ArrayList<Token>");
            return JObject::null().as_raw()
        }
    }
}