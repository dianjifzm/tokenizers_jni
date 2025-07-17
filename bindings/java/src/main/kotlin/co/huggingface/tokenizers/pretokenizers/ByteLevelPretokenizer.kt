package co.huggingface.tokenizers.pretokenizers

import co.huggingface.tokenizers.exceptions.NativeAllocationFailedException
import co.huggingface.tokenizers.exceptions.StringDecodingException
import co.huggingface.tokenizers.jni.Native

class ByteLevelPretokenizer: Pretokenizer, Native {

    private var handle: Long = -1

    init {
        this.handle = allocate()
    }

    private external fun allocate(): Long
    external override fun finalize()

    @Throws(NativeAllocationFailedException::class, StringDecodingException::class)
    external override fun pretokenize(s: String): List<String>

    external override fun decode(words: List<String>): String
}