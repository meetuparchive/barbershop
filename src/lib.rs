#[macro_use]
extern crate cpython;
#[macro_use]
extern crate lando;

// extends http::Request type with api gateway info
use lando::RequestExt;

gateway!(|request, _| {
    println!("{}", ::std::str::from_utf8(request.body())?);
    Ok(lando::Response::new(()))
});
