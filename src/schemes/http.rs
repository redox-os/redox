use programs::common::*;

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

impl SessionItem for HTTPScheme {
    fn scheme(&self) -> String {
        return "http".to_string();
    }

    fn open(&mut self, url: &URL) -> Box<Resource>{
        let mut html = "HTTP/1.1 200 OK\r\n".to_string()
                    + "Content-Type: text/html\r\n"
                    + "Connection: keep-alive\r\n"
                    + "\r\n";

        if url.path == "readme".to_string() {
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

            if url.path == "readme".to_string() {
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

            if url.path == "readme".to_string() {
                let mut resource = URL::from_string("file:///README.md".to_string()).open();

                let mut resource_data: Vec<u8> = Vec::new();
                resource.read_to_end(&mut resource_data);
                html = html + "<div class='panel panel-default'>\n".to_string();
                    if resource_data.len() > 0 {
                        html = html + "<div class='panel-heading'>\n";
                            html = html + "<h3 class='panel-title'><span class='glyphicon glyphicon-book'></span> README</h3>";
                        html = html + "</div>\n";

                        html = html + "<div class='panel-body'>\n";
                            let mut in_code = false;
                            for line in String::from_utf8(&resource_data).split("\n".to_string()){
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
                            html = html + "<h3 class='panel-title'><span class='glyphicon glyphicon-exlamation-sign'></span> Failed to open README</h3>\n";
                        html = html + "</div>\n";
                    }
                html = html + "</div>\n";
            }else{
                html = html + "<table class='table table-bordered'>\n".to_string();
                    let mut resource = URL::from_string(url.path.clone()).open();

                    let resource_type;
                    match resource.stat() {
                        ResourceType::File => resource_type = "File".to_string(),
                        ResourceType::Dir => resource_type = "Dir".to_string(),
                        ResourceType::Array => resource_type = "Array".to_string(),
                        _ => resource_type = "None".to_string()
                    }

                    html = html + "  <caption><h3>" + HTTPScheme::encode(url.path.clone()) + "</h3><h4>" + HTTPScheme::encode(resource_type) + "</h4></caption>\n";

                    let mut resource_data: Vec<u8> = Vec::new();
                    resource.read_to_end(&mut resource_data);
                    for line in String::from_utf8(&resource_data).split("\n".to_string()) {
                        html = html + "<tr><td>" + HTTPScheme::encode(line.clone()) + "</td></tr>\n";
                    }
                html = html + "</table>\n";
            }

        html = html + "</div>\n";

        return box VecResource::new(ResourceType::File, html.to_utf8());
    }
}
