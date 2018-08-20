use std::env;
use std::sync::{Mutex, Arc};
use std::collections::HashSet;
extern crate reqwest;
extern crate select;
use select::document::Document;
use select::predicate::Name;
extern crate url;
use url::Url;
use std::sync::mpsc::{channel, Sender};
extern crate threadpool;
use threadpool::ThreadPool;
extern crate num_cpus;

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
        Err(_e) => {println!("Please use an absolute url as input\nEx. http://apple.com");return},
    };
    let sites_visited: Arc<Mutex<HashSet<String>>> = Arc::new(Mutex::new(HashSet::new()));
    let (link_sender, link_receiver) = channel::<reqwest::Url>();
    //Init thread pool and get to work
    let pool = ThreadPool::new(num_cpus::get());
    link_sender.send(url).expect("Error sending url to queue");
    for link in link_receiver.iter(){
        let inner_sender = link_sender.clone();
        let inner_sites = sites_visited.clone();
        pool.execute(move|| {crawl_url(link, inner_sites, inner_sender)});
    }

}

fn crawl_url(url: reqwest::Url, sites_visited: Arc<Mutex<HashSet<String>>>, link_sender: Sender<reqwest::Url>) -> () {
    //Bad url case
    if url.cannot_be_a_base(){
        return
    }
    let mut guard = sites_visited.lock().unwrap();
    match *guard{
        ref mut hs => match hs.contains(url.as_str()){
            true => return,
            false => {hs.insert(url.clone().to_string());},
        }
    }
    drop(guard);

    println!("{}", url);

    //Request page and build vec of links on page
    let res = reqwest::get(url.as_str()).expect("error fetching url");
    let links: Vec<String> = Document::from_read(res).expect("error parsing url")
        .find(Name("a"))
        .filter_map(|n| n.attr("href"))
        .map(|x| x.to_string())
        .collect();


    //Check links and queue them for processing
    links.iter()
        .for_each(|link| {
            let recurse_url = Url::parse(&link);
            match recurse_url {
                Ok(l) => {
                    link_sender.send(l)
                    .expect("Error sending url to queue");
                },
                Err(_e) =>  {
                    let recurse_url = url.join(&link);
                    if recurse_url.is_ok(){
                        link_sender.send(recurse_url.expect("error passing recurse_url"))
                        .expect("Error sending url to queue");
                    }
                },
            }
        });
}
