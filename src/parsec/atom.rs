use parsec::{State, ParsecError, Status, Parser};
use std::fmt::{Debug, Display};
use std::sync::Arc;
use std::marker::Reflect;

pub fn one<T:'static, Index:Reflect+Debug+'static, Tran:'static>()->Parser<T, T, Index, Tran> {
    abc!(|state:&mut State<T, Index=Index, Tran=Tran>|->Status<T, Index>{
        state.next().ok_or(ParsecError::new(state.pos(), String::from("eof")))
    })
}

pub fn eq<T:'static, Index:Reflect+Debug+Display+'static, Tran:'static>(val:T)
            -> Parser<T, T, Index, Tran> where T:Eq+Display+Debug+Clone {
    abc!(move |state:&mut State<T, Index=Index, Tran=Tran>|->Status<T, Index>{
        let value = state.next();
        let pos = state.pos();
        if value.is_some() {
            let x = value.unwrap();
            if x == val {
                return Ok(x);
            }
            let val = val.clone();
            let description = format!("expect {} equal element {} at {}", val, x, pos);
            return Err(ParsecError::new(pos, description));
        }
        Err(ParsecError::new(pos, String::from("eof")))
    })
}


pub fn ne<T:'static, Index:Reflect+Debug+Display+'static, Tran:'static>(val:T)
            -> Parser<T, T, Index, Tran> where T:Display+Eq+Debug+Clone {
    abc!(move |state:&mut State<T, Index=Index, Tran=Tran>|->Status<T, Index>{
        let value = state.next();
        let pos = state.pos();
        if value.is_some() {
            let x = value.unwrap();
            if x == val {
                let val = val.clone();
                let description = format!("expect {} not equal element {} at {}", val, x, pos);
                return Err(ParsecError::new(pos, description));
            }
            return Ok(x);
        }
        Err(ParsecError::new(pos, String::from("eof")))
    })
}

pub fn eof<T:'static+Display, Index:Reflect+Debug+Display+'static, Tran:'static>()->Parser<T, (), Index, Tran> {
    abc!(|state: &mut State<T, Index=Index, Tran=Tran>|->Status<(), Index> {
        let val = state.next();
        if val.is_none() {
            Ok(())
        } else {
            let pos = state.pos();
            let description = format!("expect eof at {} but got value {}", pos, val.unwrap());
            Err(ParsecError::new(pos, description))
        }
    })
}

pub fn one_of<T:Eq+Debug+Display+Clone+'static, Index:Reflect+Debug+Display+'static, Tran:'static>(elements:&[T])
            -> Parser<T, T, Index, Tran> {
    let elements = elements.to_owned();
    abc!(move |state: &mut State<T, Index=Index, Tran=Tran>|->Status<T, Index>{
        let next = state.next();
        if next.is_none() {
            Err(ParsecError::new(state.pos(), String::from("eof")))
        } else {
            let it = next.unwrap();
            for d in &elements {
                if d == &it {
                    return Ok(it);
                }
            }
            let description = format!("<expect one of {:?} at {}, got:{}>", elements, state.pos(), it);
            Err(ParsecError::new(state.pos(), String::from(description)))
        }
    })
}

pub fn none_of<T:Eq+Debug+Display+Clone+'static, Index:Reflect+Debug+Display+'static, Tran:'static>(elements:&[T]) -> Parser<T, T, Index, Tran> {
    let elements = elements.to_owned();
    abc!(move |state: &mut State<T, Index=Index, Tran=Tran>|->Status<T, Index> {
        let next = state.next();
        if next.is_none() {
            Err(ParsecError::new(state.pos(), String::from("eof")))
        } else {
            let it = next.unwrap();
            for d in &elements {
                if d == &it {
                    let description = format!("<expect none of {:?} at {}, got:{}>", elements, state.pos(), it);
                    return Err(ParsecError::new(state.pos(), String::from(description)))
                }
            }
            Ok(it)
        }
    })
}

pub fn pack<T, R:Clone+'static, Index:Reflect+Debug+'static, Tran:'static>(element:R) -> Parser<T, R, Index, Tran> {
    abc!(move |_: &mut State<T, Index=Index, Tran=Tran>|->Status<R, Index>{
        Ok(element.clone())
    })
}

pub fn fail<T:'static+Clone, R, Index:Reflect+Debug+Display+'static, Tran:'static>(description:String) -> Parser<T, R, Index, Tran> {
    abc!(move |state:&mut State<T, Index=Index, Tran=Tran>|->Status<R, Index>{
        Err(ParsecError::new(state.pos(), String::from(description.as_str())))
    })
}
