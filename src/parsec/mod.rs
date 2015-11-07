use std::vec::Vec;
use std::iter::FromIterator;
use std::sync::Arc;
use std::boxed::Box;
use std::fmt::{Formatter, Display};
use std::fmt;
use std::clone::Clone;
use std::convert::{From};
use std::error;
use std::cmp::{min};

pub trait State<T, Tran:'static> {
    fn pos(&self)-> usize;
    fn seek_to(&mut self, usize)->bool;
    fn next(&mut self)->Option<T>;
    fn next_by(&mut self, &Fn(&T)->bool)->Status<T>;
    fn err(&self, description:String)->ParsecError {
        ParsecError::new(self.pos(), description)
    }
    fn begin(&mut self)->Tran;
    fn commit(&mut self, Tran);
    fn rollback(&mut self, Tran);
}

pub struct VecState<T> {
    index : usize,
    tran : Option<usize>,
    buffer: Vec<T>,
}

impl<A> FromIterator<A> for VecState<A> {
    fn from_iter<T>(iterator: T) -> Self where T:IntoIterator<Item=A> {
        VecState{
            index:0,
            tran:None,
            buffer:Vec::from_iter(iterator.into_iter()),
        }
    }
}

impl<T> State<T, usize> for VecState<T> where T:Clone {
    fn pos(&self) -> usize {
        self.index
    }
    fn seek_to(&mut self, to:usize) -> bool {
        if 0 as usize <= to && to < self.buffer.len() {
            self.index = to;
            true
        } else {
            false
        }
    }
    fn next(&mut self)->Option<T>{
        if 0 as usize <= self.index && self.index < self.buffer.len() {
            let item = self.buffer[self.index].clone();
            self.index += 1;
            Some(item)
        } else {
            None
        }
    }
    fn next_by(&mut self, pred:&Fn(&T)->bool)->Status<T>{
        if 0 as usize <= self.index && self.index < self.buffer.len() {
            let ref item = self.buffer[self.index];
            self.index += 1;
            if pred(item) {
                Ok(item.clone())
            } else {
                Err(self.err(String::from("predicate failed")))
            }
        } else {
            Err(self.err(String::from("eof")))
        }
    }
    fn begin(&mut self)-> usize {
        if self.tran.is_none() {
            self.tran = Some(self.index);
        } else {
            self.tran = Some(min(self.tran.unwrap(), self.index));
        }
        self.index
    }

    fn commit(&mut self, tran:usize) {
        if self.tran.is_some() {
            if self.tran.unwrap() == tran {
                self.tran = None;
            }
        }
    }

    fn rollback(&mut self, tran:usize) {
        self.index = tran;
        if self.tran.is_some() {
            if self.tran.unwrap() == tran {
                self.tran = None;
            }
        }
    }
}

pub trait Error:error::Error {
    fn pos(&self)->usize;
}

#[derive(Debug, Clone)]
pub struct ParsecError {
    _pos: usize,
    message: String,

}

impl ParsecError {
    pub fn new(pos:usize, description:String)->ParsecError{
        ParsecError{
            _pos: pos,
            message: description,
        }
    }
}

impl Error for ParsecError {
    fn pos(&self)->usize {
        self._pos
    }
}
impl error::Error for ParsecError {
    fn description(&self)->&str {
        self.message.as_str()
    }
    fn cause(&self) -> Option<&error::Error> {
        Some(self)
    }
}

impl Display for ParsecError {
    fn fmt(&self, formatter:&mut Formatter) -> Result<(), fmt::Error> {
        write!(formatter, "{}", self.message)
    }
}

//pub trait Parsec<T:'static+Clone, R:'static+Clone>:Debug where Self:Parsec<T, R, Tran:'static>+Clone+'static {
pub trait Parsec<T, R, Tran:'static> {
    fn parse(&self, &mut State<T, Tran>)->Status<R>;
}

// Type Continuation(Result) Then Pass
pub trait Monad<T:'static, R:'static, Tran:'static>:Parsec<T, R, Tran> where Self:Clone+'static, T:Clone, R:Clone {
    fn bind<P:'static+Clone>(self, binder:Arc<Box<Fn(R, &mut State<T, Tran>)->Status<P>>>)->Parser<T, P, Tran> {
        abc!(move |state:&mut State<T, Tran>|->Status<P>{
            let pre = self.parse(state);
            if pre.is_err() {
                return Err(pre.err().unwrap())
            }
            let binder = binder.clone();
            binder(pre.ok().unwrap(), state)
        })
    }
    fn then<P:'static+Clone, Thn:'static>(self, then:Thn)->Parser<T, P, Tran>
    where Thn:Parsec<T, P, Tran>+Clone{
        let then = then.clone();
        abc!(move |state:&mut State<T, Tran>|->Status<P>{
            let pre = self.parse(state);
            if pre.is_err() {
                return Err(pre.err().unwrap())
            }
            then.parse(state)
        })
    }
    fn over<P:'static+Clone, Ovr:'static>(self, over:Ovr)->Parser<T, R, Tran>
    where Ovr:Parsec<T, P, Tran>+Clone{
        let over = over.clone();
        abc!(move |state:&mut State<T, Tran>|->Status<R>{
            let re = self.parse(state);
            if re.is_err() {
                return re;
            }
            let o = over.parse(state);
            if o.is_err() {
                return Err(o.err().unwrap())
            }
            Ok(re.ok().unwrap())
        })
    }
}

pub type Status<T> = Result<T, ParsecError>;

pub type Parser<T, R, Tran:'static> = Arc<Box<Fn(&mut State<T, Tran>)->Status<R>>>;

impl<T, R, Tran:'static> Parsec<T, R, Tran> for Parser<T, R, Tran> where T:Clone, R:Clone {
    fn parse(&self, state: &mut State<T, Tran>) -> Status<R> {
        self(state)
    }
}

impl<T:'static, R:'static, Tran:'static> Monad<T, R, Tran> for Parser<T, R, Tran> where T:Clone, R:Clone {}

pub mod atom;
pub mod combinator;
// pub mod text;
