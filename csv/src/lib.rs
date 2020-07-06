mod comment;
mod python;

pub use comment::{Comment, Message, MessageBody, MessageSubject};

use csv::Reader as CsvReader;
use std::{error::Error, io::Read};

pub struct Parser<ReaderT: Read> {
    reader: CsvReader<ReaderT>,
}

impl<ReaderT: Read> Parser<ReaderT> {
    fn new(reader: ReaderT) -> Self {
        Self {
            reader: CsvReader::from_reader(reader),
        }
    }

    fn parse(&mut self) -> Result<(), Box<dyn Error>> {
        for result in self.reader.records() {
            // The iterator yields Result<StringRecord, Error>, so we check the
            // error here.
            let record = result?;
            println!("{:?}", record);
        }
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use std::fs::File;

    #[test]
    fn it_works() {
        let _fin = File::open("/home/marius/w/reinfer-csv/test.csv");
        assert_eq!(2 + 2, 4);
    }
}
