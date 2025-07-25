use std::sync::{Arc, RwLock};

use crate::models::PyModel;
use crate::tokenizer::PyAddedToken;
use pyo3::exceptions;
use pyo3::prelude::*;
use pyo3::types::*;
use serde::{Deserialize, Serialize};
use tk::models::TrainerWrapper;
use tk::Trainer;
use tokenizers as tk;

/// Base class for all trainers
///
/// This class is not supposed to be instantiated directly. Instead, any implementation of a
/// Trainer will return an instance of this class when instantiated.
#[pyclass(module = "tokenizers.trainers", name = "Trainer", subclass)]
#[derive(Clone, Deserialize, Serialize)]
#[serde(transparent)]
pub struct PyTrainer {
    pub trainer: Arc<RwLock<TrainerWrapper>>,
}

impl PyTrainer {
    #[cfg(test)]
    pub(crate) fn new(trainer: Arc<RwLock<TrainerWrapper>>) -> Self {
        PyTrainer { trainer }
    }
    pub(crate) fn get_as_subtype(&self, py: Python<'_>) -> PyResult<PyObject> {
        let base = self.clone();
        Ok(match *self.trainer.as_ref().read().unwrap() {
            TrainerWrapper::BpeTrainer(_) => Py::new(py, (PyBpeTrainer {}, base))?
                .into_pyobject(py)?
                .into_any()
                .into(),
            TrainerWrapper::WordPieceTrainer(_) => Py::new(py, (PyWordPieceTrainer {}, base))?
                .into_pyobject(py)?
                .into_any()
                .into(),
            TrainerWrapper::WordLevelTrainer(_) => Py::new(py, (PyWordLevelTrainer {}, base))?
                .into_pyobject(py)?
                .into_any()
                .into(),
            TrainerWrapper::UnigramTrainer(_) => Py::new(py, (PyUnigramTrainer {}, base))?
                .into_pyobject(py)?
                .into_any()
                .into(),
        })
    }
}
#[pymethods]
impl PyTrainer {
    fn __getstate__(&self, py: Python) -> PyResult<PyObject> {
        let data = serde_json::to_string(&self.trainer).map_err(|e| {
            exceptions::PyException::new_err(format!(
                "Error while attempting to pickle PyTrainer: {e}"
            ))
        })?;
        Ok(PyBytes::new(py, data.as_bytes()).into())
    }

    fn __setstate__(&mut self, py: Python, state: PyObject) -> PyResult<()> {
        match state.extract::<&[u8]>(py) {
            Ok(s) => {
                let unpickled = serde_json::from_slice(s).map_err(|e| {
                    exceptions::PyException::new_err(format!(
                        "Error while attempting to unpickle PyTrainer: {e}"
                    ))
                })?;
                self.trainer = unpickled;
                Ok(())
            }
            Err(e) => Err(e),
        }
    }

    fn __repr__(&self) -> PyResult<String> {
        crate::utils::serde_pyo3::repr(self)
            .map_err(|e| exceptions::PyException::new_err(e.to_string()))
    }

    fn __str__(&self) -> PyResult<String> {
        crate::utils::serde_pyo3::to_string(self)
            .map_err(|e| exceptions::PyException::new_err(e.to_string()))
    }
}

impl Trainer for PyTrainer {
    type Model = PyModel;

    fn should_show_progress(&self) -> bool {
        self.trainer.read().unwrap().should_show_progress()
    }

    fn train(&self, model: &mut PyModel) -> tk::Result<Vec<tk::AddedToken>> {
        self.trainer
            .read()
            .unwrap()
            .train(&mut model.model.write().unwrap())
    }

    fn feed<I, S, F>(&mut self, iterator: I, process: F) -> tk::Result<()>
    where
        I: Iterator<Item = S> + Send,
        S: AsRef<str> + Send,
        F: Fn(&str) -> tk::Result<Vec<String>> + Sync,
    {
        self.trainer.write().unwrap().feed(iterator, process)
    }
}

impl<I> From<I> for PyTrainer
where
    I: Into<TrainerWrapper>,
{
    fn from(trainer: I) -> Self {
        PyTrainer {
            trainer: Arc::new(RwLock::new(trainer.into())),
        }
    }
}

macro_rules! getter {
    ($self: ident, $variant: ident, $($name: tt)+) => {{
        let super_ = $self.as_ref();
        if let TrainerWrapper::$variant(ref trainer) = *super_.trainer.read().unwrap() {
            trainer.$($name)+
        } else {
            unreachable!()
        }
    }};
}

macro_rules! setter {
    ($self: ident, $variant: ident, $name: ident, $value: expr) => {{
        let super_ = $self.as_ref();
        if let TrainerWrapper::$variant(ref mut trainer) = *super_.trainer.write().unwrap() {
            trainer.$name = $value;
        }
    }};
    ($self: ident, $variant: ident, @$name: ident, $value: expr) => {{
        let super_ = $self.as_ref();
        if let TrainerWrapper::$variant(ref mut trainer) = *super_.trainer.write().unwrap() {
            trainer.$name($value);
        }
    }};
}

/// Trainer capable of training a BPE model
///
/// Args:
///     vocab_size (:obj:`int`, `optional`):
///         The size of the final vocabulary, including all tokens and alphabet.
///
///     min_frequency (:obj:`int`, `optional`):
///         The minimum frequency a pair should have in order to be merged.
///
///     show_progress (:obj:`bool`, `optional`):
///         Whether to show progress bars while training.
///
///     special_tokens (:obj:`List[Union[str, AddedToken]]`, `optional`):
///         A list of special tokens the model should know of.
///
///     limit_alphabet (:obj:`int`, `optional`):
///         The maximum different characters to keep in the alphabet.
///
///     initial_alphabet (:obj:`List[str]`, `optional`):
///         A list of characters to include in the initial alphabet, even
///         if not seen in the training dataset.
///         If the strings contain more than one character, only the first one
///         is kept.
///
///     continuing_subword_prefix (:obj:`str`, `optional`):
///         A prefix to be used for every subword that is not a beginning-of-word.
///
///     end_of_word_suffix (:obj:`str`, `optional`):
///         A suffix to be used for every subword that is a end-of-word.
///
///     max_token_length (:obj:`int`, `optional`):
///         Prevents creating tokens longer than the specified size.
///         This can help with reducing polluting your vocabulary with
///         highly repetitive tokens like `======` for wikipedia
///
#[pyclass(extends=PyTrainer, module = "tokenizers.trainers", name = "BpeTrainer")]
pub struct PyBpeTrainer {}
#[pymethods]
impl PyBpeTrainer {
    #[getter]
    fn get_vocab_size(self_: PyRef<Self>) -> usize {
        getter!(self_, BpeTrainer, vocab_size)
    }

    #[setter]
    fn set_vocab_size(self_: PyRef<Self>, vocab_size: usize) {
        setter!(self_, BpeTrainer, vocab_size, vocab_size);
    }

    #[getter]
    fn get_min_frequency(self_: PyRef<Self>) -> u64 {
        getter!(self_, BpeTrainer, min_frequency)
    }

    #[setter]
    fn set_min_frequency(self_: PyRef<Self>, freq: u64) {
        setter!(self_, BpeTrainer, min_frequency, freq);
    }

    #[getter]
    fn get_show_progress(self_: PyRef<Self>) -> bool {
        getter!(self_, BpeTrainer, show_progress)
    }

    #[setter]
    fn set_show_progress(self_: PyRef<Self>, show_progress: bool) {
        setter!(self_, BpeTrainer, show_progress, show_progress);
    }

    #[getter]
    fn get_special_tokens(self_: PyRef<Self>) -> Vec<PyAddedToken> {
        getter!(
            self_,
            BpeTrainer,
            special_tokens
                .iter()
                .map(|tok| tok.clone().into())
                .collect()
        )
    }

    #[setter]
    fn set_special_tokens(self_: PyRef<Self>, special_tokens: &Bound<'_, PyList>) -> PyResult<()> {
        setter!(
            self_,
            BpeTrainer,
            special_tokens,
            special_tokens
                .into_iter()
                .map(|token| {
                    if let Ok(content) = token.extract::<String>() {
                        Ok(tk::tokenizer::AddedToken::from(content, true))
                    } else if let Ok(mut token) = token.extract::<PyRefMut<PyAddedToken>>() {
                        token.special = true;
                        Ok(token.get_token())
                    } else {
                        Err(exceptions::PyTypeError::new_err(
                            "Special tokens must be a List[Union[str, AddedToken]]",
                        ))
                    }
                })
                .collect::<PyResult<Vec<_>>>()?
        );
        Ok(())
    }

    #[getter]
    fn get_limit_alphabet(self_: PyRef<Self>) -> Option<usize> {
        getter!(self_, BpeTrainer, limit_alphabet)
    }

    #[setter]
    fn set_limit_alphabet(self_: PyRef<Self>, limit: Option<usize>) {
        setter!(self_, BpeTrainer, limit_alphabet, limit);
    }

    #[getter]
    fn get_max_token_length(self_: PyRef<Self>) -> Option<usize> {
        getter!(self_, BpeTrainer, max_token_length)
    }

    #[setter]
    fn set_max_token_length(self_: PyRef<Self>, limit: Option<usize>) {
        setter!(self_, BpeTrainer, max_token_length, limit);
    }

    #[getter]
    fn get_initial_alphabet(self_: PyRef<Self>) -> Vec<String> {
        getter!(
            self_,
            BpeTrainer,
            initial_alphabet.iter().map(|c| c.to_string()).collect()
        )
    }

    #[setter]
    fn set_initial_alphabet(self_: PyRef<Self>, alphabet: Vec<char>) {
        setter!(
            self_,
            BpeTrainer,
            initial_alphabet,
            alphabet.into_iter().collect()
        );
    }

    #[getter]
    fn get_continuing_subword_prefix(self_: PyRef<Self>) -> Option<String> {
        getter!(self_, BpeTrainer, continuing_subword_prefix.clone())
    }

    #[setter]
    fn set_continuing_subword_prefix(self_: PyRef<Self>, prefix: Option<String>) {
        setter!(self_, BpeTrainer, continuing_subword_prefix, prefix);
    }

    #[getter]
    fn get_end_of_word_suffix(self_: PyRef<Self>) -> Option<String> {
        getter!(self_, BpeTrainer, end_of_word_suffix.clone())
    }

    #[setter]
    fn set_end_of_word_suffix(self_: PyRef<Self>, suffix: Option<String>) {
        setter!(self_, BpeTrainer, end_of_word_suffix, suffix);
    }

    #[new]
    #[pyo3(signature = (**kwargs), text_signature = None)]
    pub fn new(kwargs: Option<&Bound<'_, PyDict>>) -> PyResult<(Self, PyTrainer)> {
        let mut builder = tk::models::bpe::BpeTrainer::builder();
        if let Some(kwargs) = kwargs {
            for (key, val) in kwargs {
                let key: String = key.extract()?;
                match key.as_ref() {
                    "vocab_size" => builder = builder.vocab_size(val.extract()?),
                    "min_frequency" => builder = builder.min_frequency(val.extract()?),
                    "show_progress" => builder = builder.show_progress(val.extract()?),
                    "special_tokens" => {
                        builder = builder.special_tokens(
                            val.downcast::<PyList>()?
                                .into_iter()
                                .map(|token| {
                                    if let Ok(content) = token.extract::<String>() {
                                        Ok(PyAddedToken::from(content, Some(true)).get_token())
                                    } else if let Ok(mut token) =
                                        token.extract::<PyRefMut<PyAddedToken>>()
                                    {
                                        token.special = true;
                                        Ok(token.get_token())
                                    } else {
                                        Err(exceptions::PyTypeError::new_err(
                                            "special_tokens must be a List[Union[str, AddedToken]]",
                                        ))
                                    }
                                })
                                .collect::<PyResult<Vec<_>>>()?,
                        );
                    }
                    "limit_alphabet" => builder = builder.limit_alphabet(val.extract()?),
                    "max_token_length" => builder = builder.max_token_length(val.extract()?),
                    "initial_alphabet" => {
                        let alphabet: Vec<String> = val.extract()?;
                        builder = builder.initial_alphabet(
                            alphabet
                                .into_iter()
                                .filter_map(|s| s.chars().next())
                                .collect(),
                        );
                    }
                    "continuing_subword_prefix" => {
                        builder = builder.continuing_subword_prefix(val.extract()?)
                    }
                    "end_of_word_suffix" => builder = builder.end_of_word_suffix(val.extract()?),
                    _ => println!("Ignored unknown kwargs option {key}"),
                };
            }
        }
        Ok((PyBpeTrainer {}, builder.build().into()))
    }
}

/// Trainer capable of training a WordPiece model
///
/// Args:
///     vocab_size (:obj:`int`, `optional`):
///         The size of the final vocabulary, including all tokens and alphabet.
///
///     min_frequency (:obj:`int`, `optional`):
///         The minimum frequency a pair should have in order to be merged.
///
///     show_progress (:obj:`bool`, `optional`):
///         Whether to show progress bars while training.
///
///     special_tokens (:obj:`List[Union[str, AddedToken]]`, `optional`):
///         A list of special tokens the model should know of.
///
///     limit_alphabet (:obj:`int`, `optional`):
///         The maximum different characters to keep in the alphabet.
///
///     initial_alphabet (:obj:`List[str]`, `optional`):
///         A list of characters to include in the initial alphabet, even
///         if not seen in the training dataset.
///         If the strings contain more than one character, only the first one
///         is kept.
///
///     continuing_subword_prefix (:obj:`str`, `optional`):
///         A prefix to be used for every subword that is not a beginning-of-word.
///
///     end_of_word_suffix (:obj:`str`, `optional`):
///         A suffix to be used for every subword that is a end-of-word.
#[pyclass(extends=PyTrainer, module = "tokenizers.trainers", name = "WordPieceTrainer")]
pub struct PyWordPieceTrainer {}
#[pymethods]
impl PyWordPieceTrainer {
    #[getter]
    fn get_vocab_size(self_: PyRef<Self>) -> usize {
        getter!(self_, WordPieceTrainer, vocab_size())
    }

    #[setter]
    fn set_vocab_size(self_: PyRef<Self>, vocab_size: usize) {
        setter!(self_, WordPieceTrainer, @set_vocab_size, vocab_size);
    }

    #[getter]
    fn get_min_frequency(self_: PyRef<Self>) -> u64 {
        getter!(self_, WordPieceTrainer, min_frequency())
    }

    #[setter]
    fn set_min_frequency(self_: PyRef<Self>, freq: u64) {
        setter!(self_, WordPieceTrainer, @set_min_frequency, freq);
    }

    #[getter]
    fn get_show_progress(self_: PyRef<Self>) -> bool {
        getter!(self_, WordPieceTrainer, show_progress())
    }

    #[setter]
    fn set_show_progress(self_: PyRef<Self>, show_progress: bool) {
        setter!(self_, WordPieceTrainer, @set_show_progress, show_progress);
    }

    #[getter]
    fn get_special_tokens(self_: PyRef<Self>) -> Vec<PyAddedToken> {
        getter!(
            self_,
            WordPieceTrainer,
            special_tokens()
                .iter()
                .map(|tok| tok.clone().into())
                .collect()
        )
    }

    #[setter]
    fn set_special_tokens(self_: PyRef<Self>, special_tokens: &Bound<'_, PyList>) -> PyResult<()> {
        setter!(
            self_,
            WordPieceTrainer,
            @set_special_tokens,
            special_tokens
                .into_iter()
                .map(|token| {
                    if let Ok(content) = token.extract::<String>() {
                        Ok(tk::tokenizer::AddedToken::from(content, true))
                    } else if let Ok(mut token) = token.extract::<PyRefMut<PyAddedToken>>() {
                        token.special = true;
                        Ok(token.get_token())
                    } else {
                        Err(exceptions::PyTypeError::new_err(
                            "Special tokens must be a List[Union[str, AddedToken]]",
                        ))
                    }
                })
                .collect::<PyResult<Vec<_>>>()?
        );
        Ok(())
    }

    #[getter]
    fn get_limit_alphabet(self_: PyRef<Self>) -> Option<usize> {
        getter!(self_, WordPieceTrainer, limit_alphabet())
    }

    #[setter]
    fn set_limit_alphabet(self_: PyRef<Self>, limit: Option<usize>) {
        setter!(self_, WordPieceTrainer, @set_limit_alphabet, limit);
    }

    #[getter]
    fn get_initial_alphabet(self_: PyRef<Self>) -> Vec<String> {
        getter!(
            self_,
            WordPieceTrainer,
            initial_alphabet().iter().map(|c| c.to_string()).collect()
        )
    }

    #[setter]
    fn set_initial_alphabet(self_: PyRef<Self>, alphabet: Vec<char>) {
        setter!(
            self_,
            WordPieceTrainer,
            @set_initial_alphabet,
            alphabet.into_iter().collect()
        );
    }

    #[getter]
    fn get_continuing_subword_prefix(self_: PyRef<Self>) -> Option<String> {
        getter!(self_, WordPieceTrainer, continuing_subword_prefix().clone())
    }

    #[setter]
    fn set_continuing_subword_prefix(self_: PyRef<Self>, prefix: Option<String>) {
        setter!(self_, WordPieceTrainer, @set_continuing_subword_prefix, prefix);
    }

    #[getter]
    fn get_end_of_word_suffix(self_: PyRef<Self>) -> Option<String> {
        getter!(self_, WordPieceTrainer, end_of_word_suffix().clone())
    }

    #[setter]
    fn set_end_of_word_suffix(self_: PyRef<Self>, suffix: Option<String>) {
        setter!(self_, WordPieceTrainer, @set_end_of_word_suffix, suffix);
    }

    #[new]
    #[pyo3(
        signature = (** kwargs),
        text_signature = "(self, vocab_size=30000, min_frequency=0, show_progress=True, special_tokens=[], limit_alphabet=None, initial_alphabet= [],continuing_subword_prefix=\"##\", end_of_word_suffix=None)"
    )]
    pub fn new(kwargs: Option<&Bound<'_, PyDict>>) -> PyResult<(Self, PyTrainer)> {
        let mut builder = tk::models::wordpiece::WordPieceTrainer::builder();
        if let Some(kwargs) = kwargs {
            for (key, val) in kwargs {
                let key: String = key.extract()?;
                match key.as_ref() {
                    "vocab_size" => builder = builder.vocab_size(val.extract()?),
                    "min_frequency" => builder = builder.min_frequency(val.extract()?),
                    "show_progress" => builder = builder.show_progress(val.extract()?),
                    "special_tokens" => {
                        builder = builder.special_tokens(
                            val.downcast::<PyList>()?
                                .into_iter()
                                .map(|token| {
                                    if let Ok(content) = token.extract::<String>() {
                                        Ok(PyAddedToken::from(content, Some(true)).get_token())
                                    } else if let Ok(mut token) =
                                        token.extract::<PyRefMut<PyAddedToken>>()
                                    {
                                        token.special = true;
                                        Ok(token.get_token())
                                    } else {
                                        Err(exceptions::PyTypeError::new_err(
                                            "special_tokens must be a List[Union[str, AddedToken]]",
                                        ))
                                    }
                                })
                                .collect::<PyResult<Vec<_>>>()?,
                        );
                    }
                    "limit_alphabet" => builder = builder.limit_alphabet(val.extract()?),
                    "initial_alphabet" => {
                        let alphabet: Vec<String> = val.extract()?;
                        builder = builder.initial_alphabet(
                            alphabet
                                .into_iter()
                                .filter_map(|s| s.chars().next())
                                .collect(),
                        );
                    }
                    "continuing_subword_prefix" => {
                        builder = builder.continuing_subword_prefix(val.extract()?)
                    }
                    "end_of_word_suffix" => builder = builder.end_of_word_suffix(val.extract()?),
                    _ => println!("Ignored unknown kwargs option {key}"),
                };
            }
        }

        Ok((PyWordPieceTrainer {}, builder.build().into()))
    }
}

/// Trainer capable of training a WorldLevel model
///
/// Args:
///     vocab_size (:obj:`int`, `optional`):
///         The size of the final vocabulary, including all tokens and alphabet.
///
///     min_frequency (:obj:`int`, `optional`):
///         The minimum frequency a pair should have in order to be merged.
///
///     show_progress (:obj:`bool`, `optional`):
///         Whether to show progress bars while training.
///
///     special_tokens (:obj:`List[Union[str, AddedToken]]`):
///         A list of special tokens the model should know of.
#[pyclass(extends=PyTrainer, module = "tokenizers.trainers", name = "WordLevelTrainer")]
pub struct PyWordLevelTrainer {}
#[pymethods]
impl PyWordLevelTrainer {
    #[getter]
    fn get_vocab_size(self_: PyRef<Self>) -> usize {
        getter!(self_, WordLevelTrainer, vocab_size)
    }

    #[setter]
    fn set_vocab_size(self_: PyRef<Self>, vocab_size: usize) {
        setter!(self_, WordLevelTrainer, vocab_size, vocab_size);
    }

    #[getter]
    fn get_min_frequency(self_: PyRef<Self>) -> u64 {
        getter!(self_, WordLevelTrainer, min_frequency)
    }

    #[setter]
    fn set_min_frequency(self_: PyRef<Self>, freq: u64) {
        setter!(self_, WordLevelTrainer, min_frequency, freq);
    }

    #[getter]
    fn get_show_progress(self_: PyRef<Self>) -> bool {
        getter!(self_, WordLevelTrainer, show_progress)
    }

    #[setter]
    fn set_show_progress(self_: PyRef<Self>, show_progress: bool) {
        setter!(self_, WordLevelTrainer, show_progress, show_progress);
    }

    #[getter]
    fn get_special_tokens(self_: PyRef<Self>) -> Vec<PyAddedToken> {
        getter!(
            self_,
            WordLevelTrainer,
            special_tokens
                .iter()
                .map(|tok| tok.clone().into())
                .collect()
        )
    }

    #[setter]
    fn set_special_tokens(self_: PyRef<Self>, special_tokens: &Bound<'_, PyList>) -> PyResult<()> {
        setter!(
            self_,
            WordLevelTrainer,
            special_tokens,
            special_tokens
                .into_iter()
                .map(|token| {
                    if let Ok(content) = token.extract::<String>() {
                        Ok(tk::tokenizer::AddedToken::from(content, true))
                    } else if let Ok(mut token) = token.extract::<PyRefMut<PyAddedToken>>() {
                        token.special = true;
                        Ok(token.get_token())
                    } else {
                        Err(exceptions::PyTypeError::new_err(
                            "Special tokens must be a List[Union[str, AddedToken]]",
                        ))
                    }
                })
                .collect::<PyResult<Vec<_>>>()?
        );
        Ok(())
    }

    #[new]
    #[pyo3(signature = (**kwargs), text_signature = None)]
    pub fn new(kwargs: Option<&Bound<'_, PyDict>>) -> PyResult<(Self, PyTrainer)> {
        let mut builder = tk::models::wordlevel::WordLevelTrainer::builder();

        if let Some(kwargs) = kwargs {
            for (key, val) in kwargs {
                let key: String = key.extract()?;
                match key.as_ref() {
                    "vocab_size" => {
                        builder.vocab_size(val.extract()?);
                    }
                    "min_frequency" => {
                        builder.min_frequency(val.extract()?);
                    }
                    "show_progress" => {
                        builder.show_progress(val.extract()?);
                    }
                    "special_tokens" => {
                        builder.special_tokens(
                            val.downcast::<PyList>()?
                                .into_iter()
                                .map(|token| {
                                    if let Ok(content) = token.extract::<String>() {
                                        Ok(PyAddedToken::from(content, Some(true)).get_token())
                                    } else if let Ok(mut token) =
                                        token.extract::<PyRefMut<PyAddedToken>>()
                                    {
                                        token.special = true;
                                        Ok(token.get_token())
                                    } else {
                                        Err(exceptions::PyTypeError::new_err(
                                            "special_tokens must be a List[Union[str, AddedToken]]",
                                        ))
                                    }
                                })
                                .collect::<PyResult<Vec<_>>>()?,
                        );
                    }
                    _ => println!("Ignored unknown kwargs option {key}"),
                }
            }
        }

        Ok((
            PyWordLevelTrainer {},
            builder
                .build()
                .expect("WordLevelTrainerBuilder cannot fail")
                .into(),
        ))
    }
}

/// Trainer capable of training a Unigram model
///
/// Args:
///     vocab_size (:obj:`int`):
///         The size of the final vocabulary, including all tokens and alphabet.
///
///     show_progress (:obj:`bool`):
///         Whether to show progress bars while training.
///
///     special_tokens (:obj:`List[Union[str, AddedToken]]`):
///         A list of special tokens the model should know of.
///
///     initial_alphabet (:obj:`List[str]`):
///         A list of characters to include in the initial alphabet, even
///         if not seen in the training dataset.
///         If the strings contain more than one character, only the first one
///         is kept.
///
///     shrinking_factor (:obj:`float`):
///         The shrinking factor used at each step of the training to prune the
///         vocabulary.
///
///     unk_token (:obj:`str`):
///         The token used for out-of-vocabulary tokens.
///
///     max_piece_length (:obj:`int`):
///         The maximum length of a given token.
///
///     n_sub_iterations (:obj:`int`):
///         The number of iterations of the EM algorithm to perform before
///         pruning the vocabulary.
#[pyclass(extends=PyTrainer, module = "tokenizers.trainers", name = "UnigramTrainer")]
pub struct PyUnigramTrainer {}
#[pymethods]
impl PyUnigramTrainer {
    #[getter]
    fn get_vocab_size(self_: PyRef<Self>) -> u32 {
        getter!(self_, UnigramTrainer, vocab_size)
    }

    #[setter]
    fn set_vocab_size(self_: PyRef<Self>, vocab_size: u32) {
        setter!(self_, UnigramTrainer, vocab_size, vocab_size);
    }

    #[getter]
    fn get_show_progress(self_: PyRef<Self>) -> bool {
        getter!(self_, UnigramTrainer, show_progress)
    }

    #[setter]
    fn set_show_progress(self_: PyRef<Self>, show_progress: bool) {
        setter!(self_, UnigramTrainer, show_progress, show_progress);
    }

    #[getter]
    fn get_special_tokens(self_: PyRef<Self>) -> Vec<PyAddedToken> {
        getter!(
            self_,
            UnigramTrainer,
            special_tokens
                .iter()
                .map(|tok| tok.clone().into())
                .collect()
        )
    }

    #[setter]
    fn set_special_tokens(self_: PyRef<Self>, special_tokens: &Bound<'_, PyList>) -> PyResult<()> {
        setter!(
            self_,
            UnigramTrainer,
            special_tokens,
            special_tokens
                .into_iter()
                .map(|token| {
                    if let Ok(content) = token.extract::<String>() {
                        Ok(tk::tokenizer::AddedToken::from(content, true))
                    } else if let Ok(mut token) = token.extract::<PyRefMut<PyAddedToken>>() {
                        token.special = true;
                        Ok(token.get_token())
                    } else {
                        Err(exceptions::PyTypeError::new_err(
                            "Special tokens must be a List[Union[str, AddedToken]]",
                        ))
                    }
                })
                .collect::<PyResult<Vec<_>>>()?
        );
        Ok(())
    }

    #[getter]
    fn get_initial_alphabet(self_: PyRef<Self>) -> Vec<String> {
        getter!(
            self_,
            UnigramTrainer,
            initial_alphabet.iter().map(|c| c.to_string()).collect()
        )
    }

    #[setter]
    fn set_initial_alphabet(self_: PyRef<Self>, alphabet: Vec<char>) {
        setter!(
            self_,
            UnigramTrainer,
            initial_alphabet,
            alphabet.into_iter().collect()
        );
    }

    #[new]
    #[pyo3(
        signature = (**kwargs),
        text_signature = "(self, vocab_size=8000, show_progress=True, special_tokens=[], shrinking_factor=0.75, unk_token=None, max_piece_length=16, n_sub_iterations=2)"
    )]
    pub fn new(kwargs: Option<Bound<'_, PyDict>>) -> PyResult<(Self, PyTrainer)> {
        let mut builder = tk::models::unigram::UnigramTrainer::builder();
        if let Some(kwargs) = kwargs {
            for (key, val) in kwargs {
                let key: String = key.extract()?;
                match key.as_ref() {
                    "vocab_size" => builder.vocab_size(val.extract()?),
                    "show_progress" => builder.show_progress(val.extract()?),
                    "n_sub_iterations" => builder.n_sub_iterations(val.extract()?),
                    "shrinking_factor" => builder.shrinking_factor(val.extract()?),
                    "unk_token" => builder.unk_token(val.extract()?),
                    "max_piece_length" => builder.max_piece_length(val.extract()?),
                    "seed_size" => builder.seed_size(val.extract()?),
                    "initial_alphabet" => {
                        let alphabet: Vec<String> = val.extract()?;
                        builder.initial_alphabet(
                            alphabet
                                .into_iter()
                                .filter_map(|s| s.chars().next())
                                .collect(),
                        )
                    }
                    "special_tokens" => builder.special_tokens(
                        val.downcast::<PyList>()?
                            .into_iter()
                            .map(|token| {
                                if let Ok(content) = token.extract::<String>() {
                                    Ok(PyAddedToken::from(content, Some(true)).get_token())
                                } else if let Ok(mut token) =
                                    token.extract::<PyRefMut<PyAddedToken>>()
                                {
                                    token.special = true;
                                    Ok(token.get_token())
                                } else {
                                    Err(exceptions::PyTypeError::new_err(
                                        "special_tokens must be a List[Union[str, AddedToken]]",
                                    ))
                                }
                            })
                            .collect::<PyResult<Vec<_>>>()?,
                    ),
                    _ => {
                        println!("Ignored unknown kwargs option {key}");
                        &mut builder
                    }
                };
            }
        }

        let trainer: tokenizers::models::unigram::UnigramTrainer =
            builder.build().map_err(|e| {
                exceptions::PyException::new_err(format!("Cannot build UnigramTrainer: {e}"))
            })?;
        Ok((PyUnigramTrainer {}, trainer.into()))
    }
}

/// Trainers Module
#[pymodule]
pub fn trainers(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyTrainer>()?;
    m.add_class::<PyBpeTrainer>()?;
    m.add_class::<PyWordPieceTrainer>()?;
    m.add_class::<PyWordLevelTrainer>()?;
    m.add_class::<PyUnigramTrainer>()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tk::models::bpe::trainer::BpeTrainer;

    #[test]
    fn get_subtype() {
        Python::with_gil(|py| {
            let py_trainer = PyTrainer::new(Arc::new(RwLock::new(BpeTrainer::default().into())));
            let py_bpe = py_trainer.get_as_subtype(py).unwrap();
            assert_eq!("BpeTrainer", py_bpe.bind(py).get_type().qualname().unwrap());
        })
    }
}
