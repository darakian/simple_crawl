use std::env;
use std::sync::{Mutex, Arc};
use std::collections::HashSet;
extern crate reqwest;
extern crate select;
use select::document::Document;
use select::predicate::Name;
extern crate rayon;
use rayon::prelude::*;
extern crate url;
use url::Url;
extern crate stacker;


fn main() {

        let url = match env::args().nth(1) {
        Some(url) => url,
        None => {
            println!("Usage: client <url>");
            return;
        }
    };
    let url = match Url::parse(&url){
        Ok(token) => token,
        Err(_e) => {println!("Please use an absolute url as input\nEx. http://rescale.com");return},
    };
    let sites_visited: Arc<Mutex<HashSet<String>>> = Arc::new(Mutex::new(HashSet::new()));
    stacker::maybe_grow(32 * 1024, 1024 * 1024, || {
        match crawl_url(url.clone(), sites_visited){
            Ok(_) => (),
            Err(e) => {println!("Error crawling {}: {:?}", url, e);}
        };
    });
}

fn crawl_url(url: reqwest::Url, sites_visited: Arc<Mutex<HashSet<String>>>) -> Result<(), Box<std::error::Error>> {
    if url.cannot_be_a_base(){
        return Ok(())
    }
    let mut guard = sites_visited.lock().unwrap();
    match *guard{
        ref mut hs => match hs.contains(url.as_str()){
            true => {
                return Ok(())},
            false => {hs.insert(url.clone().to_string());},
        }
    }
    drop(guard);

    println!("{}", url);

    let res = reqwest::get(url.as_str())?;
    let links: Vec<String> = Document::from_read(res)?
        .find(Name("a"))
        .filter_map(|n| n.attr("href"))
        .map(|x| x.to_string())
        .collect();

    links.par_iter()
        .for_each(|link| {
            let recurse_url = Url::parse(&link);
            match recurse_url {
            /**/Ok(l) => {
                    stacker::maybe_grow(32 * 1024, 1024 * 1024, || {
                        match crawl_url(l, sites_visited.clone()) {
                            Ok(_) => {()},
                            Err(_e) => {return Err(Box::new(e)},
                        }
                    });
                },
            /**/Err(_e) =>  {
                    let recurse_url = url.join(&link);
                    if recurse_url.is_ok(){
                        stacker::maybe_grow(32 * 1024, 1024 * 1024, || {
                            match crawl_url(recurse_url.unwrap(), sites_visited.clone()) {
                                Ok(_) => (Ok(())),
                                Err(e) => {println!(">>> Error crawling {}: {:?}", link, e);return Err(Box::new(e))}
                            }
                        }).unwrap();
                    }
                },
            }
        });
    Ok(())
}
