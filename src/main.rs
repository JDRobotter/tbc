extern crate futures;
extern crate hyper;
extern crate hyper_tls;
extern crate tokio_core;
extern crate url;

use std::error::Error;
use std::io::{self, Write};
use futures::{Future, Stream};
use hyper::{Client, Request, Method};
use hyper_tls::HttpsConnector;

use tokio_core::reactor::{Core, Interval};
use std::time::Duration;
use url::Url;

fn main() {
    let mut core = Core::new().unwrap();
 
    // prepare an hyper client to perform HTTPS requests
    let client = Client::configure()
            .connector(HttpsConnector::new(4, &core.handle()).unwrap())
            .build(&core.handle());

    // fire an event at regular intervals to download informations
    let interval = Interval::new(Duration::new(1,0), &core.handle()).unwrap();
    let work = interval.and_then(|result| {
        println!("event");

        let mut url = Url::parse("https://data.bordeaux-metropole.fr/wfs").unwrap();
        url.query_pairs_mut()
            .clear()
            .append_pair("key","QHUHHRI7HD")
            .append_pair("REQUEST","GetFeature")
            .append_pair("SERVICE","WFS")
            .append_pair("SRSNAME","EPSG:3945")
            .append_pair("TYPENAME","SV_VEHIC_P")
            .append_pair("VERSION","1.1.0")
            .append_pair("Filter","");

        let url_str = url.as_str();
        println!("GET {}",url_str);

        let uri = url_str.parse().unwrap();
        let mut req = Request::new(Method::Get, uri);

        client.request(req).then(|result| {
            match result {
                Ok(ans) => {
                    println!("{}", ans.status());
                },
                Err(e) => println!("ERR {}", e),
            }
            Ok(())
        })
    });

    core.run(work.collect()).unwrap();
}
