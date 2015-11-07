use parsec::{State, Status, Monad, Parser, Parsec};
use parsec::atom::{pack, fail};
use std::fmt::{Debug};
use std::sync::Arc;

pub fn try<T:'static, R:'static, X:'static, Tran:'static>(p:X)->Parser<T, R, Tran>
where T:Clone, R:Clone, X:Parsec<T, R, Tran>+Clone {
    abc!(move |state: &mut State<T, Tran>|->Status<R>{
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

pub trait Or<T, R, Tran> {
    fn or(&self, Parser<T, R, Tran>)->Parser<T, R, Tran>;
}

pub type Either<T, R, Tran:'static> = Arc<Box<Fn(&mut State<T, Tran>)->Status<R>>>;
pub fn either<T, R, X:'static, Y:'static, Tran:'static>(x:X, y:Y)->Either<T, R, Tran>
where T:Clone, R:Clone, X:Parsec<T, R, Tran>+Clone, Y:Parsec<T, R, Tran>+Clone{
    let x = x.clone();
    let y = y.clone();
    abc!(move |state:&mut State<T, Tran>|->Status<R>{
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
impl<T:'static+Clone, R:'static+Clone, Tran:'static> Or<T, R, Tran> for Either<T, R, Tran> {
    fn or(&self, p:Parser<T, R, Tran>)->Parser<T, R, Tran>{
        let s:Parser<T, R, Tran> = self.clone();
        either(s, p)
    }
}

pub fn many<T:'static, R:'static, X:'static, Tran:'static>(p:X)->Parser<T, Vec<R>, Tran>
where T:Clone, R:Clone+Debug, X:Parsec<T, R, Tran>+Clone {
    let p=try(p.clone());
    abc!(move |state:&mut State<T, Tran>|->Status<Vec<R>>{
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

pub fn many1<T:'static, R:'static, X:'static, Tran:'static>(p:X)->Parser<T, Vec<R>, Tran>
where T:Clone, R:Clone+Debug, X:Parsec<T, R, Tran>+Clone {
    abc!(move |state:&mut State<T, Tran>|->Status<Vec<R>>{
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

pub fn between<T:'static, B:'static, P:'static, E:'static, X:'static, Open:'static, Close:'static, Tran:'static>
        (open:Open, close:Close, parsec:X)
        ->Parser<T, P, Tran>
where T:Clone, P:Clone, B:Clone, E:Clone, Open:Monad<T, B, Tran>+Clone, X:Parsec<T, P, Tran>+Clone,
        Close:Parsec<T, E, Tran>+Clone {
    let open = open.clone();
    let parsec = parsec.clone();
    let close = close.clone();
    abc!(move |state: &mut State<T, Tran>|->Status<P>{
        try!(open.parse(state));
        let re = parsec.parse(state);
        try!(close.parse(state));
        re
    })
}

pub fn otherwise<T:'static, R:'static, X:'static, Tran:'static>(p:X, description:String)->Parser<T, R, Tran>
where T:Clone, R:Clone, X:Parsec<T, R, Tran>+Clone {
    abc!(move |state : &mut State<T, Tran>|->Status<R>{
        either(p.clone(), fail(description.clone()).clone()).parse(state)
    })
}

pub fn many_tail<T:'static, R:'static, Tl:'static, X:'static, Tail:'static, Tran:'static>
    (p:X, tail:Tail)->Parser<T, Vec<R>, Tran>
where T:Clone, R:Clone+Debug, Tl:Clone, X:Parsec<T, R, Tran>+Clone, Tail:Parsec<T, Tl, Tran>+Clone{
    abc!(move |state:&mut State<T, Tran>|->Status<Vec<R>>{
        let p = p.clone();
        let tail = tail.clone();
        many(p).over(tail).parse(state)
    })
}

pub fn many1_tail<T:'static, R:'static, Tl:'static, X:'static, Tail:'static, Tran:'static>
    (p:X, tail:Tail)->Parser<T, Vec<R>, Tran>
where T:Clone, R:Clone+Debug, Tl:Clone, X:Monad<T, R, Tran>+Clone, Tail:Parsec<T, Tl, Tran>+Clone{
    let p = p.clone();
    let tail = tail.clone();
    abc!(move |state:&mut State<T, Tran>|->Status<Vec<R>>{
        many1(p.clone()).over(tail.clone()).parse(state)
    })
}

// We can use many/many1 as skip, but them more effective.
pub fn skip_many<T:'static, R:'static, X:'static, Tran:'static>(p:X) ->Parser<T, Vec<R>, Tran>
where T:Clone, R:Clone, X:Parsec<T, R, Tran>+Clone {
    abc!(move |state: &mut State<T, Tran>|->Status<Vec<R>>{
        let p = try(p.clone());
        loop {
            let re = p.parse(state);
            if re.is_err() {
                return Ok(Vec::new());
            }
        }
    })
}

pub fn skip_many1<T:'static, R:'static, X:'static, Tran:'static>(p:X) ->Parser<T, Vec<R>, Tran>
where T:Clone, R:Clone, X:Parsec<T, R, Tran>+Clone {
    abc!(move |state: &mut State<T, Tran>|->Status<Vec<R>>{
        let re = p.parse(state);
        if re.is_err() {
            return Err(re.err().unwrap());
        }
        skip_many(p.clone()).parse(state)
    })
}

pub fn sep_by<T:'static, Sp:'static, R:'static, Sep:'static, X:'static, Tran:'static>(parsec:X, sep:Sep)->Parser<T, Vec<R>, Tran>
where T:Clone, R:Clone+Debug, Sp:Clone, Sep:Parsec<T, Sp, Tran>+Clone, X:Parsec<T, R, Tran>+Clone {
    abc!(move |state:&mut State<T, Tran>|->Status<Vec<R>>{
        let s = try(sep.clone());
        let p = try(parsec.clone());
        either(sep_by1(p, s), pack(Vec::new())).parse(state)
    })
}

pub fn sep_by1<T:'static, Sp:'static, R:'static, Sep:'static, X:'static, Tran:'static>
        (parsec:X, sep:Sep) ->Parser<T, Vec<R>, Tran>
where T:Clone, R:Clone+Debug, Sp:Clone, Sep:Parsec<T, Sp, Tran>+Clone, X:Parsec<T, R, Tran>+Clone {
    abc!(move |state: &mut State<T, Tran>|->Status<Vec<R>>{
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
        rev.push_all(&data);
        Ok(rev)
    })
}
