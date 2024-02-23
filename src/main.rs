use lit_md::HtmlMaker;
use rocket::{http::Status, response::content::RawHtml, Request};
use std::{env, path::PathBuf};

const DOMIAIN: &str = "http://127.0.0.1:8000/";

markup::define! {
    Link<'content, 'url>(content: &'content str, url: &'url str) {
        a[href = format!("{DOMIAIN}{url}")] {
            @content
        }
    }
    Back<'url> (url: &'url str) {
        p {
            @Link {
                content: "back",
                url,
            }
        }
    }
    Head () {
        head {
            meta[name="viewport", content="width=device-width, initial-scale=1"];
            style {
                @markup::raw(r#"
                    .markdown-body {
                        box-sizing: border-box;
                        min-width: 200px;
                        max-width: 980px;
                        margin: 0 auto;
                        padding: 45px;
                    }
                    @media (max-width: 767px) {
                        .markdown-body {
                            padding: 15px;
                        }
                    }"#
                )
            }
            style {
                @markup::raw(include_str!("css/main.css"))
            }
        }
    }
    Directory <'back, 'prefix, 'basename>(back: &'back str, paths: Vec<(&'prefix str, &'basename str)>) {
        @markup::doctype()
        html {
            @Head {}
            body {
                article[class = "markdown-body"] {
                    p {
                        ul {
                            @for path in paths {
                                li {
                                    @Link {
                                        url: &format!("{}{}", path.0, path.1),
                                        content: path.1
                                    }
                                }
                            }
                        }
                    }
                    @Back {
                        url: back
                    }
                }
            }
        }
    }
    MarkDown <'back>(content: String, back: &'back str) {
        @markup::doctype()
        html {
            @Head {}
            body {
                article[class = "markdown-body"] {
                    @markup::raw(content)
                    @Back {
                        url: back
                    }
                }
            }
        }
    }
}

#[rocket::get("/<paths..>")]
async fn index(paths: PathBuf) -> Result<RawHtml<String>, Status> {
    let prefix = env::args().nth(1).unwrap();
    let md_path = PathBuf::from(prefix).join(&paths);
    let parent = paths.parent().unwrap_or(&paths);
    let back = format!("{}", parent.display());
    if md_path.is_dir() {
        let dir = md_path.read_dir().unwrap();
        let mut vec = Vec::new();
        for path in dir {
            let path = path.map_err(|_| Status::NotFound)?.file_name();
            let basename = path.into_string().map_err(|_| Status::NotFound)?;
            vec.push((
                if paths.display().to_string().is_empty() {
                    String::new()
                } else {
                    format!("{}/", paths.display())
                },
                basename,
            ));
        }
        Ok(RawHtml(
            Directory {
                back: &back,
                paths: vec
                    .iter()
                    .map(|(prefix, basename)| (&prefix[..], &basename[..]))
                    .collect(),
            }
            .to_string(),
        ))
    } else {
        let mut maker = HtmlMaker::try_from(md_path).map_err(|_| Status::NotFound)?;
        let html = maker.parse().map_err(|_| Status::NotFound)?;
        Ok(RawHtml(
            MarkDown {
                content: html,
                back: &back,
            }
            .to_string(),
        ))
    }
}

#[rocket::catch(404)]
fn not_found(req: &Request) -> RawHtml<String> {
    RawHtml(
        markup::new! {
            html {
                @Head {}
                body {
                    article[class = "markdown-body"] {
                        h1 {
                            "NOT FOUND: 404"
                        }
                        p {
                            @format!("Sorry, cannot found {}", req.uri())
                        }
                    }
                }
            }
        }
        .to_string(),
    )
}

#[rocket::launch]
fn rocket() -> _ {
    if env::args().len() < 2 {
        panic!("NOT FOUND enough args.");
    }
    rocket::build()
        .mount("/", rocket::routes![index])
        .register("/", rocket::catchers![not_found])
}
