extern crate futures;
extern crate hyper;
extern crate hyper_tls;
extern crate tokio_core;
extern crate url;
extern crate xml;

use std::error::Error;
use std::io::{self, Write};
use futures::{Future, Stream};
use hyper::{Client, Request, Method};
use hyper_tls::HttpsConnector;

use tokio_core::reactor::{Core, Interval};
use std::time::Duration;
use url::Url;

use xml::reader::{EventReader, XmlEvent};
use std::collections::HashMap;

fn parse_xml_response(body: &str) {

    println!("{}",body);
    let parser = EventReader::from_str(body);
    let mut element_name: Option<_> = None;
    let mut vehicule_name: Option<_> = None;
    let mut vehicule_speed: Option<_> = None;
    let mut vehicule_position: Option<_> = None;
    //let mut vehicules = HashMap::new();

    for e in parser {
        match e {
            Ok(XmlEvent::StartElement {name, attributes, ..}) => {
                match name.local_name.as_ref() {
                    "SV_VEHIC_P" => {
                        let value = &attributes[0].value;
                        vehicule_name = Some(value.clone());
                        vehicule_speed = None;
                        vehicule_position = None;
                    },
                    _ => {},
                };
                element_name = Some(name.local_name);
            },
            Ok(XmlEvent::EndElement {name}) => {
                match name.local_name.as_ref() {
                    "SV_VEHIC_P" => {
                        println!("{:?}: {:?} {:?}", vehicule_name, vehicule_speed, vehicule_position);
                        
                    },
                    _ => {},
                }

            },
            Ok(XmlEvent::Characters(s)) => {
                match element_name.as_ref() {
                    Some(x) => {
                        match x.as_ref() {
                            "VITESSE" => { 
                                vehicule_speed = Some(s);
                            },
                            "pos" => {
                                vehicule_position = Some(s);
                            },
                            _ => {},
                        }
                    },
                    _ => {},
                };
            },
            _ => {},
        }
    }
}

fn main() {
    let mut core = Core::new().unwrap();
 
    // prepare an hyper client to perform HTTPS requests
    let client = Client::configure()
            .connector(HttpsConnector::new(4, &core.handle()).unwrap())
            .build(&core.handle());

    // fire an event at regular intervals to download informations
    let interval = Interval::new(Duration::new(1,0), &core.handle()).unwrap();
    let work = interval
        .map_err(|_err| ())
        .and_then(|result| {

        let mut url = Url::parse("https://data.bordeaux-metropole.fr/wfs").unwrap();
        url.query_pairs_mut()
            .clear()
            .append_pair("key","QHUHHRI7HD")
            .append_pair("REQUEST","GetFeature")
            .append_pair("SERVICE","WFS")
            .append_pair("SRSNAME","EPSG:3945")
            .append_pair("TYPENAME","SV_VEHIC_P")
            .append_pair("VERSION","1.1.0")
            .append_pair("Filter","<Filter>\
                <AND>\
                <PropertyIsEqualTo>\
                    <PropertyName>RS_SV_LIGNE_A</PropertyName>\
                    <Literal>59</Literal>\
                </PropertyIsEqualTo>\
                <PropertyIsEqualTo>\
                    <PropertyName>SENS</PropertyName>\
                    <Literal>ALLER</Literal>\
                </PropertyIsEqualTo>\
                </AND>\
                </Filter>");
        let url_str = url.as_str();
        println!("GET {}",url_str);

        let uri = url_str.parse().unwrap();
        let mut req = Request::new(Method::Get, uri);

        client.request(req)
            .map_err(|_err| ())
            .and_then(|answer| {

            answer.body().map_err(|_err| ()).concat2().then(|r| {
                match r {
                    Ok(body) => {
                        let body = String::from_utf8_lossy(&body.to_vec()).to_string();
                        parse_xml_response(&body)
                    },
                    Err(e) => {},
                };
                Ok(())
            })
        })
    });

    core.run(work.collect()).unwrap();
}
