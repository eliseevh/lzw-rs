#[macro_use]
extern crate rocket;

use rocket::{fs::relative, http::ContentType, Data};
use server::*;
use std::io;

async fn process_file(
    content_type: &ContentType,
    data: Data<'_>,
    process: impl FnOnce(&mut io::BufReader<std::fs::File>, &mut Vec<u8>) -> io::Result<()>,
    name_gen: impl FnOnce(String) -> String,
) -> io::Result<FileResponse> {
    let (file, out_path) = get_file_from_multipart_data(content_type, data, name_gen).await?;

    let mut file = io::BufReader::new(file);
    let mut out = Vec::new();

    process(&mut file, &mut out)?;

    FileResponse::build(out_path, out)
}

#[post("/compress", data = "<data>")]
async fn compress_file(content_type: &ContentType, data: Data<'_>) -> io::Result<FileResponse> {
    process_file(content_type, data, lzw::encode, |name| {
        format!("{name}.compress")
    })
    .await
}

#[post("/decompress", data = "<data>")]
async fn decompress_file(content_type: &ContentType, data: Data<'_>) -> io::Result<FileResponse> {
    process_file(content_type, data, lzw::decode, |name| {
        name.trim_end_matches(".compress").into()
    })
    .await
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/", routes![compress_file, decompress_file])
        .mount("/", rocket::fs::FileServer::from(relative!("static")))
}
