use parsec::{State, Status, Monad, Parser, Parsec};
use parsec::atom::{pack, fail};
use std::fmt::{Debug, Display};
use std::sync::Arc;
use std::marker::Reflect;
use std::cmp::PartialEq;

pub fn try<T:'static, R:'static, X:'static, Index:Reflect+Debug+'static, Tran:'static>
        (p:X)->Parser<T, R, Index, Tran>
where T:Clone, R:Clone, X:Parsec<T, R, Index=Index, Tran=Tran>+Clone {
    abc!(move |state: &mut State<T, Index=Index, Tran=Tran>|->Status<R, Index>{
        let tran = state.begin();
        let res = p.parse(state);
        if res.is_ok() {
            state.commit(tran);
        } else {
            state.rollback(tran);
        }
        res
    })
}

pub trait Or<T, R, Index:Reflect+Debug, Tran> {
    fn or(&self, Parser<T, R, Index, Tran>)->Parser<T, R, Index, Tran>;
}

pub type Either<T, R, Index:'static, Tran:'static> = Arc<Box<Fn(&mut State<T, Index=Index, Tran=Tran>)->Status<R, Index>>>;
pub fn either<T, R, X:'static, Y:'static, Index:PartialEq+Reflect+Debug+'static, Tran:'static>
        (x:X, y:Y)->Either<T, R, Index, Tran>
where T:Clone, R:Clone, X:Parsec<T, R, Index=Index, Tran=Tran>+Clone,
            Y:Parsec<T, R, Index=Index, Tran=Tran>+Clone{
    let x = x.clone();
    let y = y.clone();
    abc!(move |state:&mut State<T, Index=Index, Tran=Tran>|->Status<R, Index>{
        let pos = state.pos();
        let val = x.parse(state);
        if val.is_ok() {
            val
        } else {
            if pos == state.pos() {
                y.parse(state)
            } else {
                val
            }
        }
    })
}
impl<T:'static+Clone, R:'static+Clone, Index:PartialEq+Reflect+Debug+'static, Tran:'static> Or<T, R, Index, Tran> for Either<T, R, Index, Tran> {
    fn or(&self, p:Parser<T, R, Index, Tran>)->Parser<T, R, Index, Tran>{
        let s:Parser<T, R, Index, Tran> = self.clone();
        either(s, p)
    }
}

pub fn many<T:'static, R:'static, X:'static, Index:Reflect+Debug+'static, Tran:'static>(p:X)->Parser<T, Vec<R>, Index, Tran>
where T:Clone, R:Clone+Debug, X:Parsec<T, R, Index=Index, Tran=Tran>+Clone {
    let p=try(p.clone());
    abc!(move |state:&mut State<T, Index=Index, Tran=Tran>|->Status<Vec<R>, Index>{
        let mut re = Vec::<R>::new();
        loop {
            let r = p.parse(state);
            if r.is_ok() {
                re.push(r.unwrap());
            } else {
                break;
            }
        }
        Ok(re.clone())
    })
}

pub fn many1<T:'static, R:'static, X:'static, Index:Reflect+Debug+'static, Tran:'static>(p:X)->Parser<T, Vec<R>, Index, Tran>
where T:Clone, R:Clone+Debug, X:Parsec<T, R, Index=Index, Tran=Tran>+Clone {
    abc!(move |state:&mut State<T, Index=Index, Tran=Tran>|->Status<Vec<R>, Index>{
        let first = try!(p.parse(state));
        let mut re = Vec::new();
        re.push(first);
        let psc = try(p.clone());
        loop {
            let r = psc.parse(state);
            if r.is_ok() {
                re.push(r.unwrap());
            } else {
                break;
            }
        }
        Ok(re.clone())
    })
}

pub fn between<T:'static, B:'static, P:'static, E:'static, X:'static, Open:'static, Close:'static,
            Index:Reflect+Debug+'static, Tran:'static>
        (open:Open, close:Close, parsec:X)
        ->Parser<T, P, Index, Tran>
where T:Clone, P:Clone, B:Clone, E:Clone, Open:Monad<T, B, Index=Index, Tran=Tran>+Clone,
        X:Parsec<T, P, Index=Index, Tran=Tran>+Clone,
        Close:Parsec<T, E, Index=Index, Tran=Tran>+Clone {
    let open = open.clone();
    let parsec = parsec.clone();
    let close = close.clone();
    abc!(move |state: &mut State<T, Index=Index, Tran=Tran>|->Status<P, Index>{
        try!(open.parse(state));
        let re = parsec.parse(state);
        try!(close.parse(state));
        re
    })
}

pub fn otherwise<T:'static, R:'static, X:'static, Index:PartialEq+Reflect+Debug+Display+'static, Tran:'static>(p:X, description:String)->Parser<T, R, Index, Tran>
where T:Clone, R:Clone, X:Parsec<T, R, Index=Index, Tran=Tran>+Clone {
    abc!(move |state : &mut State<T, Index=Index, Tran=Tran>|->Status<R, Index>{
        either(p.clone(), fail(description.clone()).clone()).parse(state)
    })
}

pub fn many_till<T:'static, R:'static, Tl:'static, X:'static, Till:'static, Index:Reflect+Debug+'static, Tran:'static>
    (p:X, till:Till)->Parser<T, Vec<R>, Index, Tran>
where T:Clone, R:Clone+Debug, Tl:Clone, X:Parsec<T, R, Index=Index, Tran=Tran>+Clone,
            Till:Parsec<T, Tl, Index=Index, Tran=Tran>+Clone{
    abc!(move |state:&mut State<T, Index=Index, Tran=Tran>|->Status<Vec<R>, Index>{
        let p = p.clone();
        let end = try(till.clone());
        let mut re = Vec::<R>::new();
        loop {
            let stop = end.parse(state);
            if stop.is_ok() {
                return Ok(re.clone());
            } else {
                let item = try!(p.parse(state));
                re.push(item);
            }
        }
    })
}

// We can use many/many1 as skip, but them more effective.
pub fn skip<T:'static, R:'static, X:'static, Index:PartialEq+Reflect+Debug+'static, Tran:'static>
        (p:X) ->Parser<T, Vec<R>, Index, Tran>
where T:Clone, R:Clone, X:Parsec<T, R, Index=Index, Tran=Tran>+Clone {
    abc!(move |state: &mut State<T, Index=Index, Tran=Tran>|->Status<Vec<R>, Index>{
        let p = try(p.clone());
        loop {
            let re = p.parse(state);
            if re.is_err() {
                return Ok(Vec::new());
            }
        }
    })
}

pub fn skip1<T:'static, R:'static, X:'static, Index:PartialEq+Reflect+Debug+'static, Tran:'static>
        (p:X) ->Parser<T, Vec<R>, Index, Tran>
where T:Clone, R:Clone, X:Parsec<T, R, Index=Index, Tran=Tran>+Clone {
    abc!(move |state: &mut State<T, Index=Index, Tran=Tran>|->Status<Vec<R>, Index>{
        try!(p.parse(state));
        skip(p.clone()).parse(state)
    })
}

pub fn sep_by<T:'static, Sp:'static, R:'static, Sep:'static, X:'static, Index:PartialEq+Reflect+Debug+'static, Tran:'static>
        (parsec:X, sep:Sep)->Parser<T, Vec<R>, Index, Tran>
where T:Clone, R:Clone+Debug, Sp:Clone, Sep:Parsec<T, Sp, Index=Index, Tran=Tran>+Clone,
            X:Parsec<T, R, Index=Index, Tran=Tran>+Clone {
    abc!(move |state:&mut State<T, Index=Index, Tran=Tran>|->Status<Vec<R>, Index>{
        let s = try(sep.clone());
        let p = try(parsec.clone());
        either(sep_by1(p, s), pack(Vec::new())).parse(state)
    })
}

pub fn sep_by1<T:'static, Sp:'static, R:'static, Sep:'static, X:'static, Index:PartialEq+Reflect+Debug+'static, Tran:'static>
        (parsec:X, sep:Sep) ->Parser<T, Vec<R>, Index, Tran>
where T:Clone, R:Clone+Debug, Sp:Clone, Sep:Parsec<T, Sp, Index=Index, Tran=Tran>+Clone,
        X:Parsec<T, R, Index=Index, Tran=Tran>+Clone {
    abc!(move |state: &mut State<T, Index=Index, Tran=Tran>|->Status<Vec<R>, Index>{
        let parsec = parsec.clone();
        let x = parsec.parse(state);
        if x.is_err() {
            return Err(x.err().unwrap());
        }
        let mut rev = Vec::new();
        let head = x.ok().unwrap();
        let tail = sep_by(parsec.clone(), sep.clone()).parse(state);
        let data = tail.unwrap();
        rev.push(head);
        rev.extend_from_slice(&data);
        Ok(rev)
    })
}
