use actix_files::{Files, Directory};
use actix_web::{HttpServer, App, HttpRequest, HttpResponse, dev::ServiceResponse};

fn human_readable_size(size_bytes: u64) -> String {
    let units = ["B", "KB", "MB", "GB", "TB", "PB", "EB"];
    let mut size = size_bytes as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < units.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    let mut formatted = format!("{:.2}", size);

    if formatted.ends_with(".00") {
        formatted.truncate(formatted.len() - 3);
    } else if formatted.ends_with("0") {
        formatted.truncate(formatted.len() - 1);
    }

    format!("{} {}", formatted, units[unit_index])
}

fn renderer(directory: &Directory, req: &HttpRequest) -> Result<ServiceResponse, std::io::Error> {
    let req_path = req.path();

    let mut html = String::new();

    html.push_str(&format!("<!DOCTYPE html>
<html lang=\"en\">
<head>
    <meta charset=\"UTF-8\">
    <title>Directory Listing for {req_path} - mirrors.doleckijakub.pl</title>
    <style>
        body {{
            font-family: sans-serif;
            color: #e8e8e8;
            padding: 2em;
            background-color: #1f1f1f;
        }}
        
        a {{
            text-decoration: none;
            color: #7acc00;
        }}

        a:hover {{
            text-decoration: underline;
        }}

        table {{
            width: 100%;
            border-collapse: collapse;
            table-layout: auto;
            margin-top: 1em;
        }}

        th, td {{
            padding: 10px;
            border-bottom: 1px solid #555;
            white-space: nowrap;
        }}

        td:first-child {{
            text-align: left;
            overflow: hidden;
            text-overflow: ellipsis;
            white-space: nowrap;
            min-width: 100%;
            display: block;
        }}

        td:not(first-child) {{
            text-align: right;
            width: 1px;
            white-space: nowrap;
        }}
    </style>
</head>
<body>
    <h1>Directory Listing for {req_path}</h1>
    <table>
        <thead>
            <tr>
                <th>Name</th>
                <th>Size</th>
                <th>Last modified</th>
            </tr>
        </thead>
        <tbody>"));

    let mut entries = std::fs::read_dir(directory.path.clone())?
        .filter_map(|entry| entry.ok())
        .collect::<Vec<_>>();

    entries.sort_by_key(|entry| entry.file_name());
    entries.sort_by_key(|entry| !entry.file_type().expect("entry.file_type()").is_dir());

    for entry in entries {
        let metadata = entry.metadata()?;
        let file_name = entry.file_name().to_string_lossy().to_string();
        let file_type = entry.file_type()?;

        let is_dir = file_type.is_dir();
        let is_symlink = file_type.is_symlink();

        let display_file_name = if is_dir {
            format!("{file_name}/")
        } else if is_symlink {
            format!("@{file_name}")
        } else {
            file_name.clone()
        };

        let size_bytes = if is_dir {
            fs_extra::dir::get_size(entry.path()).expect("fs_extra::dir::get_size")
        } else {
            metadata.len()
        };

        let size = human_readable_size(size_bytes);

        let modified = metadata.modified().expect("metadata.modified()");
        let modified_date = chrono::DateTime::<chrono::Utc>::from(modified)
            .format("%Y-%m-%d %H:%M:%S UTC")
            .to_string();

        html.push_str(&format!(
            "<tr>
                <td><a href=\"{file_name}\">{display_file_name}</a></td>
                <td>{size}</td>
                <td>{modified_date}</td>
            </tr>"
        ));
    }

    html.push_str("</tbody>
    </table>
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
                    .redirect_to_slash_directory()
                    .files_listing_renderer(renderer)
                )
    })
    .bind("127.0.0.1:8006")?
    .run()
    .await
}

