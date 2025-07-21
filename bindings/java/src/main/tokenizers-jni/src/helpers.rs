use jni::JNIEnv;
use jni::objects::JValue;
use jni::sys::{jint, jobject};




pub fn string_vector_to_arraylist(_env: &mut JNIEnv, vector: &Vec<String>) -> Result<jobject, String>{
    match _env.new_object("java/util/ArrayList", "(I)V", &[JValue::Int(vector.len() as jint)]){
        Ok(jarray_) => {
            return Ok(jarray_.as_raw());
        },
        Err(_e) => return Err("Unable to allocate java.util.ArrayList".to_string())
    };
}