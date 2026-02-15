use crate::cook::ident;
use crate::recipe::SourceRecipe;
use crate::web::get_category;
use crate::{cook::tree::format_size, recipe::CookRecipe};
use pkg::Package;
use std::collections::BTreeMap;
use std::{fs, path::Path};

pub fn generate_html_pkg(
    package: &Package,
    recipe: &CookRecipe,
    dependents: &Vec<String>,
    stage_files: &Option<String>,
    html_path: &Path,
    config: &crate::web::CliWebConfig,
) {
    let name = &package.name;
    let version = &package.version;
    let target = &package.target;
    let category = &get_category(&recipe.dir);
    let description = recipe
        .recipe
        .package
        .description
        .as_ref()
        .map(|p| p.as_str())
        .unwrap_or("-");

    let desc_html = recipe
        .recipe
        .package
        .description
        .as_ref()
        .map(|desc| format!(r#"<p class="description">{}</p>"#, desc))
        .unwrap_or_default();

    let repo_url = &config.repo_url;

    let deps_html = if package.depends.is_empty() {
        String::from("<p>None</p>")
    } else {
        let items: Vec<String> = package
            .depends
            .iter()
            .map(|dep| format!(r#"<li><a href="{dep}.html">{dep}</a></li>"#))
            .collect();
        format!("<ul>\n{}\n</ul>", items.join("\n"))
    };

    let dependents_html = if dependents.is_empty() {
        String::from("<p>None</p>")
    } else {
        let items: Vec<String> = dependents
            .iter()
            .map(|dep| format!(r#"<li><a href="{dep}.html">{dep}</a></li>"#))
            .collect();
        format!("<ul>\n{}\n</ul>", items.join("\n"))
    };

    let mut source_html = match &recipe.recipe.source {
        Some(SourceRecipe::Git { git, .. }) => {
            let host = get_hostname(git);
            let tree_link = get_tree_url(git, host, &package.source_identifier, None);
            let short_commit = &package.source_identifier[0..7];
            format!(
                r#"
<table>
    <tr><th>Git:</th><td><a href="{git}" target="_blank">{host}</a></td></tr>
    <tr><th>Commit:</th><td><a href="{tree_link}" target="_blank">{short_commit}</a></td></tr>
</table>"#
            )
        }
        Some(SourceRecipe::Tar { tar, .. }) => {
            let host = get_hostname(tar);
            format!(
                r#"<table>
    <tr><th>Tarball:</th><td><a href="{tar}" target="_blank">{host}</a></td></tr>
</table>"#
            )
        }
        Some(SourceRecipe::SameAs { same_as }) => {
            let r = Path::new(same_as).file_name().unwrap().to_string_lossy();
            format!(
                r#"<table>
    <tr><th>Same as:</th><td><a href="{r}.html">{r}</a></td></tr>
</table>"#
            )
        }
        _ => String::from(r#"<p>No source specified.</p>"#),
    };

    let (files_html, files_count) = if let Some(stage_files) = stage_files {
        let count = stage_files
            .split('\n')
            .filter(|p| !p.ends_with('/') && !p.is_empty())
            .count();
        (format!("<pre>{stage_files}</pre>"), format!("{}", count))
    } else {
        (
            String::from(r#"<p>No package files defined.</p>"#),
            String::from("?"),
        )
    };

    {
        let host = get_hostname(&config.this_repo);
        let tree_link = get_tree_url(
            &config.this_repo,
            host,
            &package.commit_identifier,
            Some(&format!("recipes/{category}/{name}/recipe.toml")),
        );
        let short_commit = &package.commit_identifier[0..7];
        source_html += &format!(
            r#"
<table>
    <tr><th>Build script:</th><td><a href="{tree_link}" target="_blank">{short_commit}</a></td></tr>
</table>
"#
        );
    }

    let (arch, os) = {
        let target_split: Vec<&str> = package.target.split('-').collect();
        (
            target_split
                .get(0)
                .map(|s| s.to_string())
                .unwrap_or("-".into()),
            target_split
                .get(2)
                .map(|s| s.to_string())
                .unwrap_or("-".into()),
        )
    };

    let html = format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{name} - Redox OS Package</title>
    <link rel="stylesheet" href="style.css">
</head>
<body>
    <header class="pkg-header">
        <div class="container">
            <a href="index.html" class="back-link">&larr; Back to packages</a>
            <h1>{name} <span class="version">{version}</span></h1>
{desc_html}
            <p class="description">{description}</p>
            <div class="install-action">
                <span class="prompt">$</span>
                <code>pkg install {name}</code>
            </div>
        </div>
    </header>
    <main class="pkg-content container">
        <div class="pkg-main">
            <section class="pkg-deps card">
                <h2>Dependencies</h2>
{deps_html}
            </section>
            <section class="pkg-dependents card">
                <h2>Dependents</h2>
{dependents_html}
            </section>
            <section class="pkg-recipe card">
                <h2>Package Files</h2>
{files_html}
            </section>
        </div>
        
        <section class="pkg-meta card">
            <table>
                <tr><th></th><td><a href="{repo_url}/{target}/{name}.pkgar" target="_blank">Download</a></td></tr>
            </table>
            <h2>Package Info</h2>
            <table>
                <tr><th>OS</th><td>{os}</td></tr>
                <tr><th>Architecture</th><td>{arch}</td></tr>
                <tr><th>Category</th><td><a href="index.html#cat-{category}">{category}</a></td></tr>
                <tr><th>Network Size</th><td>{network_size}</td></tr>
                <tr><th>Storage Size</th><td>{storage_size}</td></tr>
                <tr><th>File count</th><td>{files_count}</td></tr>
                <tr><th>Published</th><td title="{published}">{published_short}</td></tr>
                <tr><th>Hash</th><td><code class="hash meta-box">{blake3}</code></td></tr>
            </table>
            <h2>Package Source</h2>
{source_html}
            <div style="height:100px"></div>
        </section>
    </main>
</body>
</html>"#,
        network_size = format_size(package.network_size),
        storage_size = format_size(package.storage_size),
        published_short = &package.time_identifier[0..10],
        published = package.time_identifier,
        blake3 = package.blake3,
    );

    fs::write(html_path, html).expect("Failed to write package HTML file");
}

pub fn generate_html_index(
    grouped_packages: BTreeMap<String, Vec<&(Package, CookRecipe)>>,
    index_path: &Path,
    config: &crate::web::CliWebConfig,
) {
    let mut categories_html = Vec::new();

    for (category, pkgs) in grouped_packages {
        let cards_html: Vec<String> = pkgs
            .iter()
            .map(|(pkg, _recipe)| {
                let name = &pkg.name;
                format!(
                    r#"
<div class="package-card">
    <h3 class="pkg-name"><a href="{name}.html">{name}</a></h3>
    <div class="pkg-stats">
        <span class="pkg-version">{version}</span>
        <span class="pkg-size">{size}</span>
    </div>
</div>"#,
                    name = name,
                    version = pkg.version,
                    size = format_size(pkg.network_size)
                )
            })
            .collect();

        let category_block = format!(
            r#"
<section class="category-section">
    <h2 class="category-title" id="cat-{category}">{category}</h2>
    <div class="package-grid">
{cards}
    </div>
</section>"#,
            category = category,
            cards = cards_html.join("\n")
        );

        categories_html.push(category_block);
    }

    let html = format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Redox Package Repository</title>
    <link rel="stylesheet" href="style.css">
</head>
<body>
    <header class="index-header">
        <h1>Redox OS Package Repository</h1>
        <p class="description">Repository for <code>{target}</code></p>
    </header>

    <main class="index-content container">
{category_sections}

    <footer>
        <p>Generated at <code>{commit_time}</code> with build tree <a href="{commit_tree}" target="_blank">{commit_hash}</a></p>
        <div style="height:100px"></div>
    </footer>
    </main>
</body>
</html>"#,
        target = redoxer::target(),
        category_sections = categories_html.join("\n\n"),
        commit_time = &ident::get_ident().time,
        commit_hash = &ident::get_ident().commit[0..7],
        commit_tree = get_tree_url(
            &config.this_repo,
            get_hostname(&config.this_repo),
            &ident::get_ident().commit,
            None
        ),
    );

    println!("Generating web content to {}", index_path.display());
    fs::write(index_path, html).expect("Failed to write index HTML file");
}

fn get_hostname(url: &str) -> &str {
    url.split("://")
        .nth(1)
        .unwrap_or(url)
        .split('/')
        .next()
        .unwrap_or(url)
        .split(':')
        .next()
        .unwrap_or(url)
}

pub fn get_tree_url(git_url: &str, host: &str, commit: &str, folder: Option<&str>) -> String {
    let mut base_url = git_url.trim_end_matches(".git").to_string();

    if let Some(ssh_path) = base_url.strip_prefix("git@") {
        // "git@github.com:user/repo" -> "https://github.com/user/repo"
        base_url = format!("https://{}", ssh_path.replace(':', "/"));
    } else if base_url.starts_with("git://") {
        // "git://github.com/user/repo" -> "https://github.com/user/repo"
        base_url = base_url.replacen("git://", "https://", 1);
    }

    let base_url = if host == "github.com" {
        format!("{}/tree/{}", base_url, commit)
    } else if host.contains("gitlab") {
        format!("{}/-/tree/{}", base_url, commit)
    } else {
        return format!("{}?commit={}", base_url, commit);
    };

    match folder {
        Some(f) => format!("{base_url}/{f}"),
        None => base_url,
    }
}
