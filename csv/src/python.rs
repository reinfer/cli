use pyo3::{
    exceptions::ValueError,
    prelude::*,
    types::{IntoPyDict, PyDict},
};
use std::fs::File;

use crate::{Comment, Parser};

impl IntoPyDict for Comment {
    fn into_py_dict(self, py: Python) -> &PyDict {
        let dict = PyDict::new(py);
        comment_to_dict(self, dict).expect("Failed to set_item on dict");
        dict
    }
}

fn comment_to_dict(comment: Comment, dict: &PyDict) -> PyResult<()> {
    dict.set_item("id", comment.id)?;
    Ok(())
}

#[pyclass]
struct FileParser {
    parser: Parser<File>,
}

#[pymethods]
impl FileParser {
    #[new]
    fn new(path: String) -> PyResult<Self> {
        let file = File::open(path)?;
        Ok(Self {
            parser: Parser::new(file),
        })
    }

    fn do_it(&mut self) -> PyResult<()> {
        self.parser
            .parse()
            .map_err(|error| PyErr::new::<ValueError, _>(error.to_string()))?;
        Ok(())
    }
}

impl Drop for FileParser {
    fn drop(&mut self) {
        eprintln!("Drop");
    }
}

#[pymodule(reinfer_csv)]
fn rust2py(_py: Python, py_module: &PyModule) -> PyResult<()> {
    py_module.add_class::<FileParser>()?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use pyo3::prelude::*;
    use pyo3::types::PyDict;

    #[test]
    fn test_print() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let locals = PyDict::new(py);

        py.run("foo = 'one'; print(foo)", None, Some(locals))
            .expect("smash");
        py.run("print(\"hello python\")", None, None).expect("bang");

        println!("hello {}", locals.get_item("foo").unwrap());
    }
}
