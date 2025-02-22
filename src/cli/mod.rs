use std::io;

pub(crate) mod list;
pub(crate) mod provision;

pub(crate) trait Execute {
    fn execute(self) -> io::Result<()>;
}
