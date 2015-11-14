use std::vec::Vec;
use std::iter::FromIterator;
use std::sync::Arc;
use std::boxed::Box;
use std::fmt::{Formatter, Debug, Display};
use std::fmt;
use std::clone::Clone;
use std::convert::{From};
use std::error;
use std::marker::Reflect;

pub trait State<T> {
    type Index:Reflect+Debug;
    type Tran;
    fn pos(&self)-> Self::Index;
    fn seek_to(&mut self, Self::Index)->bool;
    fn next(&mut self)->Option<T>;
    fn next_by(&mut self, &Fn(&T)->bool)->Status<T, Self::Index>;
    fn err(&self, description:String)->ParsecError<Self::Index> {
        ParsecError::new(self.pos(), description)
    }
    fn begin(&mut self)->Self::Tran;
    fn commit(&mut self, Self::Tran);
    fn rollback(&mut self, Self::Tran);
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

impl<T> State<T> for VecState<T> where T:Clone {
    type Index = usize;
    type Tran = usize;
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
    fn next_by(&mut self, pred:&Fn(&T)->bool)->Status<T, usize>{
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
    type Index;
    fn pos(&self)->Self::Index;
}

#[derive(Debug, Clone)]
pub struct ParsecError<Index:Debug+Reflect+'static> {
    _pos: Index,
    message: String,
}

impl<Index:Debug+Reflect+'static> ParsecError<Index> {
    pub fn new(pos:Index, description:String)->ParsecError<Index>{
        ParsecError{
            _pos: pos,
            message: description,
        }
    }
}

impl<Index:Debug+Reflect+Clone+'static> Error for ParsecError<Index> {
    type Index = Index;
    fn pos(&self)->Index {
        let p = self._pos.clone();
        p
    }
}

impl<Index:Debug+Reflect+'static> error::Error for ParsecError<Index> {
    fn description(&self)->&str {
        self.message.as_str()
    }
    fn cause(&self) -> Option<&error::Error> {
        Some(self)
    }
}

impl<Index:Reflect+Debug> Display for ParsecError<Index> {
    fn fmt(&self, formatter:&mut Formatter) -> Result<(), fmt::Error> {
        write!(formatter, "{}", self.message)
    }
}

//pub trait Parsec<T:'static+Clone, R:'static+Clone>:Debug where Self:Parsec<T, R, Tran:'static>+Clone+'static {
pub trait Parsec<T, R> {
    type Index:Reflect+Debug;
    type Tran;
    fn parse(&self, &mut State<T, Index=Self::Index, Tran=Self::Tran>)->Status<R, Self::Index>;
}

// Type Continuation(Result) Then Pass
pub trait Monad<T:'static, R:'static>:Parsec<T, R>
        where Self:Clone+'static, T:Clone, R:Clone {
    fn bind<P:'static+Clone>(self, binder:Arc<Box<Fn(R, &mut State<T, Index=Self::Index, Tran=Self::Tran>)
                ->Status<P, Self::Index>>>)->Parser<T, P, Self::Index, Self::Tran> {
        abc!(move |state:&mut State<T, Index=Self::Index, Tran=Self::Tran>|->Status<P, Self::Index>{
            let pre = self.parse(state);
            if pre.is_err() {
                return Err(pre.err().unwrap())
            }
            let binder = binder.clone();
            binder(pre.ok().unwrap(), state)
        })
    }
    fn then<P:'static+Clone, Thn:'static>(self, then:Thn)->Parser<T, P, Self::Index, Self::Tran>
    where Thn:Parsec<T, P, Index=Self::Index, Tran=Self::Tran>+Clone {
        let then = then.clone();
        abc!(move |state:&mut State<T, Index=Self::Index, Tran=Self::Tran>|->Status<P, Thn::Index>{
            let pre = self.parse(state);
            if pre.is_err() {
                return Err(pre.err().unwrap())
            }
            then.parse(state)
        })
    }
    fn over<P:'static+Clone, Ovr:'static>(self, over:Ovr)->Parser<T, R, Self::Index, Self::Tran>
    where Ovr:Parsec<T, P, Index=Self::Index, Tran=Self::Tran>+Clone{
        let over = over.clone();
        abc!(move |state:&mut State<T, Index=Self::Index, Tran=Self::Tran>|->Status<R, Self::Index>{
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

pub type Status<T, Index> = Result<T, ParsecError<Index>>;

pub type Parser<T, R, Index, Tran> = Arc<Box<Fn(&mut State<T, Index=Index, Tran=Tran>)->Status<R, Index>>>;

impl<T, R, Index:Debug+Reflect+'static, Tran:'static> Parsec<T, R> for Parser<T, R, Index, Tran> where T:Clone, R:Clone {
    type Index=Index;
    type Tran=Tran;
    fn parse(&self, state: &mut State<T, Index=Index, Tran=Tran>) -> Status<R, Index> {
        self(state)
    }
}

impl<T:'static, R:'static, Index:Debug+Reflect+'static, Tran:'static> Monad<T, R> for Parser<T, R, Index, Tran> where T:Clone, R:Clone {}

pub mod atom;
pub mod combinator;
// pub mod text;
