pub struct WriteIoToFmt<W: std::io::Write> {
    write: W,
    error: Option<std::io::Error>,
}

impl<W: std::io::Write> WriteIoToFmt<W> {
    pub fn new(write: W) -> Self {
        Self { write, error: None }
    }

    pub fn error(self) -> Option<std::io::Error> {
        self.error
    }
}

impl<W: std::io::Write> std::fmt::Write for WriteIoToFmt<W> {
    fn write_str(&mut self, s: &str) -> Result<(), std::fmt::Error> {
        if self.error.is_some() {
            return Err(std::fmt::Error);
        }
        match self.write.write_all(s.as_bytes()) {
            Ok(()) => Ok(()),
            Err(error) => {
                self.error = Some(error);
                Err(std::fmt::Error)
            }
        }
    }
}
