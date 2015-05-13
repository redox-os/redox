use core::clone::Clone;

use common::memory::*;
use common::random::*;
use common::string::*;

use drivers::disk::*;

use filesystems::unfs::*;

pub fn http_response(request: String) -> String{
    let mut method = "GET".to_string();
    let mut path = "/".to_string();
    let mut version = "HTTP/1.1".to_string();

    for row in request.split("\r\n".to_string()) {
        let mut i = 0;
        for col in row.split(" ".to_string()) {
            match i {
                0 => method = col,
                1 => path = col,
                2 => version = col,
                _ => ()
            }
            i += 1;
        }
        break;
    }

    let mut html = "HTTP/1.1 200 OK\r\n".to_string()
                + "Content-Type: text/html\r\n"
                + "Connection: close\r\n"
                + "\r\n";

    if path == "/files".to_string() {
        html = html + "<title>Files - Redox</title>\r\n";
    }else if path == "/readme".to_string() {
        html = html + "<title>Readme - Redox</title>\r\n";
    }else{
        html = html + "<title>Home - Redox</title>\r\n";
    }
    html = html + "<link rel='stylesheet' href='https://maxcdn.bootstrapcdn.com/bootstrap/3.3.4/css/bootstrap.min.css'>\r\n";
    html = html + "<link rel='stylesheet' href='https://maxcdn.bootstrapcdn.com/bootstrap/3.3.4/css/bootstrap-theme.min.css'>\r\n";
    html = html + "<script src='https://maxcdn.bootstrapcdn.com/bootstrap/3.3.4/js/bootstrap.min.js'></script>\r\n";

    html = html + "<div class='container'>\r\n";
        html = html + "<nav class='navbar navbar-default'>\r\n";
        html = html + "  <div class='container-fluid'>\r\n";
        html = html + "    <div class='navbar-header'>\r\n";
        html = html + "      <button type='button' class='navbar-toggle collapsed' data-toggle='collapse' data-target='#navbar-collapse'></button>\r\n";
        html = html + "      <a class='navbar-brand' href='/'>Redox Web Interface</a>\r\n";
        html = html + "    </div>\r\n";
        html = html + "    <div class='collapse navbar-collapse' id='navbar-collapse'>\r\n";
        html = html + "      <ul class='nav navbar-nav navbar-right'>\r\n";

        if path == "/files".to_string() {
            html = html + "        <li><a href='/'>Home</a></li>\r\n";
            html = html + "        <li class='active'><a href='/files'>Files</a></li>\r\n";
            html = html + "        <li><a href='/readme'>Readme</a></li>\r\n";
        }else if path == "/readme".to_string() {
            html = html + "        <li><a href='/'>Home</a></li>\r\n";
            html = html + "        <li><a href='/files'>Files</a></li>\r\n";
            html = html + "        <li class='active'><a href='/readme'>Readme</a></li>\r\n";
        }else{
            html = html + "        <li class='active'><a href='/'>Home</a></li>\r\n";
            html = html + "        <li><a href='/files'>Files</a></li>\r\n";
            html = html + "        <li><a href='/readme'>Readme</a></li>\r\n";
        }

        html = html + "      </ul>\r\n";
        html = html + "    </div>\r\n";
        html = html + "  </div>\r\n";
        html = html + "</nav>\r\n";

        if path == "/files".to_string() {
            unsafe{
                let unfs = UnFS::new(Disk::new());

                html = html + "<table class='table table-bordered'>\r\n";
                    html = html + "  <caption><h3>Files</h3></caption>\r\n";
                    html = html + "<taFiles:<br/>\r\n";
                    let files = unfs.list();
                    for file in files.as_slice() {
                        html = html + "  <tr><td>" + file.clone() + "</td></tr>\r\n";
                    }
                html = html + "</table>\r\n";
            }
        }else if path == "/readme".to_string() {
            unsafe {
                html = html + "<div class='panel panel-default'>\r\n";
                    let unfs = UnFS::new(Disk::new());
                    let readme_file = "README.md".to_string();
                    let readme_c_str = unfs.load(readme_file.clone());
                    if readme_c_str > 0 {
                        let readme = String::from_c_str(readme_c_str as *const u8);
                        unalloc(readme_c_str);

                        html = html + "<div class='panel-heading'>\r\n";
                            html = html + "<h3 class='panel-title'><span class='glyphicon glyphicon-book'></span> " + readme_file.clone() + "</h3>";
                        html = html + "</div>\r\n";

                        html = html + "<div class='panel-body'>\r\n";
                            let mut in_code = false;
                            for line in readme.split("\n".to_string()){
                                if line.starts_with("# ".to_string()){
                                    html = html + "<h1>" + line.substr(2, line.len() - 2) + "</h1>\r\n";
                                }else if line.starts_with("## ".to_string()){
                                    html = html + "<h2>" + line.substr(3, line.len() - 3) + "</h2>\r\n";
                                }else if line.starts_with("### ".to_string()){
                                    html = html + "<h3>" + line.substr(4, line.len() - 4) + "</h3>\r\n";
                                }else if line.starts_with("- ".to_string()){
                                    html = html + "<li>" + line.substr(2, line.len() - 2) + "</li>\r\n";
                                }else if line.starts_with("```".to_string()){
                                    if in_code {
                                        html = html + "</pre>\r\n";
                                        in_code = false;
                                    }else{
                                        html = html + "<pre>\r\n";
                                        in_code = true;
                                    }
                                }else{
                                    html = html + line;
                                    if in_code {
                                        html = html + "\r\n";
                                    }else{
                                        html = html + "<br/>\r\n";
                                    }
                                }
                            }
                            if in_code {
                                html = html + "</pre>\r\n";
                            }
                        html = html + "</div>\r\n";
                    }else{
                        html = html + "<div class='panel-heading'>\r\n";
                            html = html + "<h3 class='panel-title'><span class='glyphicon glyphicon-exlamation-sign'></span> Failed to open " + readme_file.clone() + "</h3>";
                        html = html + "</div>\r\n";
                    }
                html = html + "</div>\r\n";
            }
        }else{
            html = html + "<table class='table table-bordered'>\r\n";
                html = html + "  <caption><h3>Request</h3></caption>\r\n";
                html = html + "  <tr><th>Key</th><th>Value</th></tr>\r\n";
                let mut first = true;
                for row in request.split("\r\n".to_string()) {
                    if row.len() > 0 {
                        if first {
                            first = false;
                        }else{
                            html = html + "  <tr>";
                            for column in row.split(": ".to_string()) {
                                html = html + "<td>" + column + "</td>";
                            }
                            html = html + "</tr>\r\n";
                        }
                    }
                }
            html = html + "</table>\r\n";
        }

        html = html + "<ul class='list-group'>\r\n";
            html = html + "<li class='list-group-item'><h4 class='list-group-item-heading'>Server Information</h4></li>\r\n";
            html = html + "<li class='list-group-item'>Method: " + method + "</li>\r\n";
            html = html + "<li class='list-group-item'>Path: " + path.clone() + "</li>\r\n";
            html = html + "<li class='list-group-item'>Version: " + version + "</li>\r\n";
            html = html + "<li class='list-group-item'>Random Number: " + rand() + "</li>\r\n";
            html = html + "<li class='list-group-item'>Memory Used: " + memory_used()/1024/1024 + " MB</li>\r\n";
            html = html + "<li class='list-group-item'>Memory Free: " + memory_free()/1024/1024 + " MB</li>\r\n";
        html = html + "</ul>\r\n";

    html = html + "</div>\r\n";
    return html;
}
