use std::io;

pub(crate) mod list;

pub(crate) trait Execute {
    fn execute(self) -> io::Result<()>;
}
