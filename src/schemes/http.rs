use core::clone::Clone;

use redox_alloc::boxed::*;

use common::memory::*;
use common::string::*;
use common::url::*;

use filesystems::unfs::*;

use programs::session::*;

pub struct HTTPScheme;

impl HTTPScheme {
    pub fn encode(text: String) -> String{
        let mut html = String::new();

        for c in text.chars() {
            match c {
                '&' => html = html + "&amp;",
                '"' => html = html + "&quot;",
                '<' => html = html + "&lt;",
                '>' => html = html + "&gt;",
                _ => html = html + c
            }
        }

        return html;
    }
}

impl SessionModule for HTTPScheme {
    fn scheme(&self) -> String {
        return "http".to_string();
    }

    fn on_url(&mut self, session: &Session, url: &URL, callback: Box<Fn(String)>){
        let mut path = String::new();

        for part in url.path.iter() {
            path = path + "/" + part.clone();
        }

        let mut html = "HTTP/1.1 200 OK\r\n".to_string()
                    + "Content-Type: text/html\r\n"
                    + "Connection: keep-alive\r\n"
                    + "\r\n";

        if path == "/files".to_string() {
            html = html + "<title>Files - Redox</title>\n";
        }else if path == "/readme".to_string() {
            html = html + "<title>Readme - Redox</title>\n";
        }else{
            html = html + "<title>Home - Redox</title>\n";
        }
        html = html + "<link rel='icon' href='data:;base64,iVBORw0KGgo='>\n";
        html = html + "<link rel='stylesheet' href='https://maxcdn.bootstrapcdn.com/bootstrap/3.3.4/css/bootstrap.min.css'>\n";
        html = html + "<link rel='stylesheet' href='https://maxcdn.bootstrapcdn.com/bootstrap/3.3.4/css/bootstrap-theme.min.css'>\n";
        html = html + "<script src='https://maxcdn.bootstrapcdn.com/bootstrap/3.3.4/js/bootstrap.min.js'></script>\n";

        html = html + "<div class='container'>\n";
            html = html + "<nav class='navbar navbar-default'>\n";
            html = html + "  <div class='container-fluid'>\n";
            html = html + "    <div class='navbar-header'>\n";
            html = html + "      <button type='button' class='navbar-toggle collapsed' data-toggle='collapse' data-target='#navbar-collapse'></button>\n";
            html = html + "      <a class='navbar-brand' href='/'>Redox Web Interface</a>\n";
            html = html + "    </div>\n";
            html = html + "    <div class='collapse navbar-collapse' id='navbar-collapse'>\n";
            html = html + "      <ul class='nav navbar-nav navbar-right'>\n";

            if path == "/readme".to_string() {
                html = html + "        <li><a href='/'>Home</a></li>\n";
                html = html + "        <li class='active'><a href='/readme'>Readme</a></li>\n";
            }else{
                html = html + "        <li class='active'><a href='/'>Home</a></li>\n";
                html = html + "        <li><a href='/readme'>Readme</a></li>\n";
            }

            html = html + "      </ul>\n";
            html = html + "    </div>\n";
            html = html + "  </div>\n";
            html = html + "</nav>\n";

            if path == "/readme".to_string() {
                unsafe {
                    html = html + "<div class='panel panel-default'>\n";
                        let unfs = UnFS::new();
                        let readme_file = "README.md".to_string();
                        let readme_c_str = unfs.load(readme_file.clone());
                        if readme_c_str > 0 {
                            let readme = String::from_c_str(readme_c_str as *const u8);
                            unalloc(readme_c_str);

                            html = html + "<div class='panel-heading'>\n";
                                html = html + "<h3 class='panel-title'><span class='glyphicon glyphicon-book'></span> " + readme_file.clone() + "</h3>";
                            html = html + "</div>\n";

                            html = html + "<div class='panel-body'>\n";
                                let mut in_code = false;
                                for line in readme.split("\n".to_string()){
                                    if line.starts_with("# ".to_string()){
                                        html = html + "<h1>" + HTTPScheme::encode(line.substr(2, line.len() - 2)) + "</h1>\n";
                                    }else if line.starts_with("## ".to_string()){
                                        html = html + "<h2>" + HTTPScheme::encode(line.substr(3, line.len() - 3)) + "</h2>\n";
                                    }else if line.starts_with("### ".to_string()){
                                        html = html + "<h3>" + HTTPScheme::encode(line.substr(4, line.len() - 4)) + "</h3>\n";
                                    }else if line.starts_with("- ".to_string()){
                                        html = html + "<li>" + HTTPScheme::encode(line.substr(2, line.len() - 2)) + "</li>\n";
                                    }else if line.starts_with("```".to_string()){
                                        if in_code {
                                            html = html + "</pre>\n";
                                            in_code = false;
                                        }else{
                                            html = html + "<pre>\n";
                                            in_code = true;
                                        }
                                    }else{
                                        html = html + HTTPScheme::encode(line);
                                        if in_code {
                                            html = html + "\n";
                                        }else{
                                            html = html + "<br/>\n";
                                        }
                                    }
                                }
                                if in_code {
                                    html = html + "</pre>\n";
                                }
                            html = html + "</div>\n";
                        }else{
                            html = html + "<div class='panel-heading'>\n";
                                html = html + "<h3 class='panel-title'><span class='glyphicon glyphicon-exlamation-sign'></span> Failed to open " + readme_file.clone() + "</h3>\n";
                            html = html + "</div>\n";
                        }
                    html = html + "</div>\n";
                }
            }else{
                let url_string = path.substr(1, path.len());
                if url_string.len() > 0 {
                    let url = URL::from_string(url_string);
                    let data = String::new(); //session.on_url(&url);
                    html = html + "<table class='table table-bordered'>\n";
                        html = html + "  <caption><h3>" + HTTPScheme::encode(url.to_string()) + "</h3></caption>\n";
                        for line in data.split("\n".to_string()) {
                            html = html + "<tr><td>" + HTTPScheme::encode(line.clone()) + "</td></tr>\n";
                        }
                    html = html + "</table>\n";
                }else{
                    html = html + "<table class='table table-bordered'>\n";
                        html = html + "  <caption><h3>Schemes</h3></caption>\n";
                        for module in session.modules.iter() {
                            let scheme = module.scheme();
                            if scheme.len() > 0 {
                                html = html + "<tr><td><a href='/" + scheme.clone() + ":///'>" + scheme.clone() + "</a></td></tr>";
                            }
                        }
                    html = html + "</table>\n";
                }
            }
        html = html + "</div>\n";

        callback(html);
    }
}
