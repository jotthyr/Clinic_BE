#[macro_use] extern crate rocket;
extern crate serde_json;
extern crate rustc_serialize;

use rocket::serde::{Serialize, Deserialize};
use rocket::serde::json::{Json};

use std::fs::File;
use std::io::{Write};
use std::io::Read;

use rocket::http::Header;
use rocket::{Request, Response};
use rocket::fairing::{Fairing, Info, Kind};

use rocket::http::ContentType;
use rocket::http::Status;
use rocket::http::Method;

#[derive(Clone)]
#[derive(Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
struct Data {
    name: String,
    details: Vec<String>,
    price: String,
}

pub struct CORS;

#[rocket::async_trait]
impl Fairing for CORS {
    fn info(&self) -> Info {
        Info {
            name: "Attaching CORS headers to responses",
            kind: Kind::Response
        }
    }

    async fn on_response<'r>(&self, request: &'r Request<'_>, response: &mut Response<'r>) {
        response.set_header(Header::new("Access-Control-Allow-Origin", "*"));
        response.set_header(Header::new("Access-Control-Allow-Methods", "POST, GET, PATCH, OPTIONS"));
        response.set_header(Header::new("Access-Control-Allow-Headers", "*"));
        response.set_header(Header::new("Access-Control-Allow-Credentials", "true"));

        if request.method() == Method::Options {
            let body = "";
            response.set_header(ContentType::Plain);
            response.set_sized_body(body.len(), std::io::Cursor::new(body));
            response.set_status(Status::Ok);
        }
    }
}

#[catch(default)]
fn default(status: Status, req: &Request) -> String {
    format!("{} ({})", status, req.uri())
}

#[catch(500)]
fn internal_error() -> &'static str {
    "Whoops! Looks like we messed up."
}

#[catch(404)]
fn not_found(req: &Request<'_>) -> String {
    if req.method() == Method::Options {
        return "".to_string();
    }
    format!("Sorry, '{}' is not a valid path.", req.uri())
}

#[post("/post-json-data", format = "json", data = "<test_data>")]
fn post_data(test_data: Json<Vec<Data>>) -> std::io::Result<()> {

    let mut f = File::create("test.json").expect("Failed to create");

    let mut d: Vec<Data> = Vec::new();

    for value in test_data.iter() {
        d.push(Data {
                name: value.name.to_owned(),
                details: value.details.to_owned(),
                price: value.price.to_owned(),
        })
    };

    f.write_all(serde_json::to_string(&d.clone()).unwrap().as_bytes()).expect("Failed to write");

    Ok(())

}

#[get("/get-json-data")]
fn get_data() -> std::io::Result<Option<String>> {

    let mut file = File::open("test.json").unwrap();
    let mut data = String::new();
    file.read_to_string(&mut data).unwrap();

    let json_data = rustc_serialize::json::Json::from_str(&data).unwrap();

    std::io::Result::Ok(Some(json_data.to_string()))

}

#[launch]
fn rocket() -> _ {
    rocket::build().attach(CORS).mount("/", routes![get_data, post_data]).register("/", catchers![internal_error, not_found, default])
}
