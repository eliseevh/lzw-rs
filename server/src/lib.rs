use bytes::Bytes;
use rocket::{
    http::{ContentType, Header, Status},
    response::{Responder, Result},
    Data, Request, Response,
};
use rocket_multipart_form_data::{
    MultipartFormData, MultipartFormDataField, MultipartFormDataOptions,
};
use std::{fs, io};

const FILE_SIZE_LIMIT: u64 = 64 * 1024 * 1024; // 64MB

fn get_file_parser_options() -> MultipartFormDataOptions<'static> {
    MultipartFormDataOptions::with_multipart_form_data_fields(vec![MultipartFormDataField::file(
        "file",
    )
    .size_limit(FILE_SIZE_LIMIT)])
}

pub async fn get_file_from_multipart_data(
    content_type: &ContentType,
    data: Data<'_>,
    name_gen: impl FnOnce(String) -> String,
) -> io::Result<(fs::File, String)> {
    let data = MultipartFormData::parse(content_type, data, get_file_parser_options())
        .await
        .map_err(|error| io::Error::new(io::ErrorKind::Other, error))?;

    let file = &data.files.get("file").ok_or(io::Error::new(
        io::ErrorKind::Other,
        "No file named \"file\" in data",
    ))?[0];

    let out_file_path = name_gen(file.file_name.clone().ok_or(io::Error::new(
        io::ErrorKind::Other,
        "Unnamed uploaded file",
    ))?);

    fs::File::open(&file.path).map(|file| (file, out_file_path))
}

pub struct FileResponse {
    data: io::Cursor<Bytes>,
    name: String,
}

impl FileResponse {
    pub fn build(name: String, data: Vec<u8>) -> io::Result<Self> {
        Ok(Self {
            name,
            data: io::Cursor::new(data.into()),
        })
    }
}

impl<'r, 'o: 'r> Responder<'r, 'o> for FileResponse {
    fn respond_to(self, _: &'r Request<'_>) -> Result<'o> {
        Ok(Response::build()
            .status(Status::Ok)
            .header(Header::new(
                "Content-Disposition",
                format!("attachment; filename=\"{}\"", self.name),
            ))
            .header(ContentType::Bytes)
            .streamed_body(self.data)
            .finalize())
    }
}
