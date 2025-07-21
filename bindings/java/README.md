### build

```bash
# 编译
cargo build --release

# 检查库依赖
ldd target/release/libtokenizers_jni.so

# 函数导出名
nm -D target/release/libtokenizers_jni.so
```


