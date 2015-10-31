use parsec::{State, Status, Parsec, Monad, Parser, parser};
use parsec::combinator::{either, try, many1};
use parsec::atom::{OneOf, pack, eq, one_of};
use std::sync::Arc;
use std::boxed::Box;

pub struct StringState {
    buffer:String,
    index:usize,
}

impl StringState {
    pub fn new(data:String)->StringState {
        StringState{buffer:data, index:0}
    }
}

impl State<char> for StringState {
    fn pos(&self)-> usize {
        self.index
    }
    fn seek_to(&mut self, to:usize)->bool{
        if 0 as usize <= to && to < self.buffer.len() {
            self.index = to;
            true
        } else {
            false
        }
    }
    fn next(&mut self)->Option<char> {
        let next= self.index+1;
        let re = self.buffer.chars().nth(next);
        if re.is_some() {
            self.index = next;
        }
        re
    }
    fn next_by(&mut self, pred:&Fn(&char)->bool)->Status<char> {
        let data = self.next();
        if data.is_none() {
            return Err(self.err(String::from("eof")));
        } else {
            let item = data.unwrap();
            if pred(&item) {
                return Ok(item.clone());
            }
        }
        return Err(self.err(String::from("predicate failed")));
    }
}

pub fn space() -> OneOf<char> {
    one_of(&vec![' ', '\t'])
}

pub fn white_space() -> Parser<char, char> {
    abc!(|state: &mut State<char>| -> Status<char>{
        state.next_by(&|x:&char| x.is_whitespace())
    })
}

pub fn newline() -> Parser<char, String> {
    abc!(|state: &mut State<char>| -> Status<String>{
        let rel = eq('\r');
        let nl = eq('\n');
        let thn = either(try(nl.clone()).then(pack(String::from("\r\n"))),
                                pack(String::from("\r")));
        either(rel.then(thn.clone()), nl.then(pack(String::from("\n")))).parse(state)
    })
}

pub fn digit() -> Parser<char, char> {
    abc!(|state: &mut State<char>| -> Status<char>{
        state.next_by(&|x:&char| x.is_numeric())
    })
}

pub fn alpha() -> Parser<char, char> {
    abc!(|state: &mut State<char>| -> Status<char>{
        state.next_by(&|x:&char| x.is_alphabetic())
    })
}

pub fn alphanumeric() -> Parser<char, char> {
    abc!(|state: &mut State<char>| -> Status<char>{
        state.next_by(&|x:&char| x.is_alphanumeric())
    })
}

pub fn control() -> Parser<char, char> {
    abc!(|state: &mut State<char>| -> Status<char>{
        state.next_by(&|x:&char| x.is_control())
    })
}

pub fn uinteger() -> Parser<char, String> {
    abc!(|state: &mut State<char>|-> Status<String> {
        let data = try!(many1(digit)(state))
        Ok(.iter().cloned().collect::<String>())
    })
}

pub fn integer() ->Parser<char, String>{
    abc!(|state: &mut State<char>|->Status<String>{
        let mut re = String::from("");
        if try(eq('-'))(state).is_ok() {
            re.push_str("-");
        }
        let x = try!(uinteger()(state));
        re.push_str(x.as_str());
        Ok(re)
    })
}

pub fn ufloat() -> Parser<char, String> {
    abc!(|state: &mut State<char>|->Status<String>{
        let mut re = try!(either(uinteger(), pack(String::from("0")))(state));
        try!(eq('.')(state));
        let x = try!(uinter(state));
        re.push_str(x);
        Ok(re)
    })
}

pub fn float() -> Parser<char, String>{
    abc!(|state:&mut State<char>|->Status<String>{
        let mut re = String::from("");
        if try(eq('-'))(state).is_ok() {
            re.push_str("-");
        }
        x = ufloat()(state)
        re.push_str(x)
        Ok(re)
    })
}
