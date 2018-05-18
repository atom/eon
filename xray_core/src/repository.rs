use buffer::Buffer;
use cross_platform::{Path, PathComponent};
use futures::Future;
use std::cell::RefCell;
use std::rc::Rc;

pub trait Repository {
    fn open(&self, path: &Path) -> Box<Future<Item = Rc<RefCell<Buffer>>, Error = OpenError>>;
    fn paths(&self) -> Box<Cursor>;
}

pub trait LocalRepository: Repository {
    fn path(&self) -> &Path;
    fn ready(&self) -> Box<Future<Item = (), Error = InitError>>;
}

pub trait Cursor {
    fn name(&self) -> Option<&PathComponent>;
    fn descend(&mut self);
    fn ascend(&mut self);
    fn next_sibling(&mut self);
}

pub enum InitError {}
pub enum OpenError {}
