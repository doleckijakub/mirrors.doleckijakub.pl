use actix_files::{Files, Directory};
use actix_web::{HttpServer, App, HttpRequest, HttpResponse, dev::ServiceResponse};

fn renderer(directory: &Directory, req: &HttpRequest) -> Result<ServiceResponse, std::io::Error> {
    let req_path = req.path();

    let mut html = String::new();

    html.push_str(&format!("<!DOCTYPE html>
<html lang=\"en\">
<head>
    <meta charset=\"UTF-8\">
    <title>Directory Listing for {req_path} - mirrors.doleckijakub.pl</title>
    <style>
        body {{ font-family: sans-serif; padding: 2em; background-color: #1f1f1f; }}
        h1 {{ color: #e8e8e8; }}
        ul {{ list-style: none; padding: 0; }}
        li {{ padding: 0.5em 0; }}
        a {{ text-decoration: none; color: #7acc00; }}
        a:hover {{ text-decoration: underline; }}
    </style>
</head>
<body>
    <h1>Directory Listing for {req_path}</h1>
    <ul>"));

    let entries = std::fs::read_dir(directory.path.clone())?;

    for entry in entries {
        let entry = entry?;
        let file_name = entry.file_name();
        let file_name = file_name.to_string_lossy();

        let full_path = if req_path == "/" {
            format!("/{}", file_name)
        } else {
            format!("{}/{}", req_path, file_name)
        };

        html.push_str(&format!(
            "<li><a href=\"{}\">{}</a></li>",
            full_path, file_name
        ));
    }

    html.push_str("</ul>
</body>
</html>");

    Ok(ServiceResponse::new(req.clone(), HttpResponse::Ok().body(html)))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .service(
                Files::new("/", "/srv/mirrors")
                    .show_files_listing()
                    .prefer_utf8(true)
                    .files_listing_renderer(renderer)
                )
    })
    .bind("127.0.0.1:8081")?
    .run()
    .await
}

