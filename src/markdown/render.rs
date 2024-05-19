use super::parser::Rule;
use comrak::{markdown_to_html, Options};
use pest::iterators::{Pair, Pairs};
use regex::RegexBuilder;

pub fn from_tree(tree: &Pairs<'_, Rule>, mut original_in: String) -> String {
    let mut options = Options::default();

    options.extension.table = true;
    options.extension.superscript = true;
    options.extension.strikethrough = true;
    options.extension.autolink = true;
    options.extension.header_ids = Option::Some(String::new());
    // options.render.unsafe_ = true;
    options.render.escape = true;
    options.parse.smart = false;

    // escape < and >
    // original_in = regex_replace(&original_in, "<", "&lt;");
    // original_in = regex_replace(&original_in, ">", "&gt;");

    // underline
    original_in = regex_replace(
        &original_in,
        "(\\_{2})(.*?)(\\_{2})",
        "<span style=\"text-decoration: underline;\" role=\"underline\">$2</span>",
    );

    // image with sizing
    let image_sizing_regex = RegexBuilder::new("(!)\\[(.*?)\\]\\((.*?)\\)\\:\\{(.*?)x(.*?)\\}")
        .multi_line(true)
        .build()
        .unwrap();

    for capture in image_sizing_regex.captures_iter(&original_in.clone()) {
        let title = capture.get(2).unwrap().as_str();
        let src = capture.get(3).unwrap().as_str();

        let width = capture.get(4).unwrap().as_str();
        let height = capture.get(5).unwrap().as_str();

        let result = &format!("<img alt=\"{title}\" title=\"{title}\" src=\"{src}\" style=\"width: {width}px; height: {height}px;\" />");
        original_in = original_in.replace(capture.get(0).unwrap().as_str(), result);
    }

    // admonitions
    original_in = regex_replace(
        // title and content
        &original_in,
        "^(\\!{3})\\s(?<TYPE>.*?)\\s(?<TITLE>.+)\\n(?<CONTENT>.+)$",
        "<div class=\"mdnote note-$2\">
            <b class=\"mdnote-title\">$3</b>
            <p>$4</p>
        </div>\n",
    );

    original_in = regex_replace(
        // title only
        &original_in,
        "^(\\!{3})\\s(?<TYPE>.*?)\\s(?<TITLE>.*?)$",
        "<div class=\"mdnote note-$2\"><b class=\"mdnote-title\">$3</b></div>\n",
    );

    // ...
    let mut out: String = markdown_to_html(&original_in, &options);
    out = regex_replace(&out, "(&!)(.*?);", "&$2;");
    out = out.replace("&quot;", "\"");

    // css ">"
    out = out.replace("&gt; \\.", "> .");
    out = out.replace("&gt; #", "> #");
    out = out.replace("(&gt;", "(>");
    out = out.replace(" &gt; ", " > ");

    // ...
    for block in tree.clone().into_iter() {
        let btype = block.as_rule();
        let block_string = block.as_span().as_str().to_string();
        let inner = block.into_inner().collect::<Vec<Pair<'_, Rule>>>();

        // e#theme (identifier)#
        if btype == Rule::THEME {
            let theme = inner.get(0).unwrap();

            out = out.replace(
                &format!("e#{}#", block_string),
                &format!("<theme>{}</theme>", theme.as_span().as_str()),
            );

            continue;
        }

        // e#hsl (hue/lit/sat) (percentage/int)#
        if btype == Rule::HSL {
            let which = inner.get(0).unwrap();
            let value = inner.get(1).unwrap();

            out = out.replace(
                &format!("e#{}#", block_string),
                &format!(
                    "<{}>{}</{}>",
                    which.as_span().as_str(),
                    value.as_span().as_str(),
                    which.as_span().as_str()
                ),
            );

            continue;
        }

        // e#html (identifier) {attrs}#
        if btype == Rule::HTML {
            let tag = inner.get(0).unwrap();

            let attrs = inner
                .iter()
                .skip(1)
                .into_iter()
                .collect::<Vec<&Pair<'_, Rule>>>();

            // build attrs string
            let mut attrs_string = String::new();

            for attr in attrs {
                attrs_string += &format!("{} ", attr.as_span().as_str());
            }

            // replace
            out = out.replace(
                &format!("e#{}#", block_string.replace("\"", "&quot;")),
                &format!("<{} {}>", tag.as_span().as_str(), attrs_string),
            );

            continue;
        }

        // e#chtml (identifier)#
        if btype == Rule::CHTML {
            let tag = inner.get(0).unwrap();

            // replace
            out = out.replace(
                &format!("e#{}#", block_string),
                &format!("</{}>", tag.as_span().as_str()),
            );

            continue;
        }

        // e#id (identifier)#
        if btype == Rule::ID {
            let id = inner.get(0).unwrap();

            // replace
            out = out.replace(
                &format!("e#{}#", block_string),
                &format!("<span id=\"{}\">", id.as_span().as_str()),
            );

            continue;
        }

        // e#class (identifier)+#
        if btype == Rule::CLASS {
            let attrs = inner.into_iter().collect::<Vec<Pair<'_, Rule>>>();

            // build attrs string
            let mut attrs_string = String::new();

            for attr in attrs {
                attrs_string += &format!("{} ", attr.as_span().as_str());
            }

            // replace
            out = out.replace(
                &format!("e#{}#", block_string.replace("\"", "&quot;")),
                &format!("<span class=\"{}\">", attrs_string),
            );

            continue;
        }

        // e#close#
        if btype == Rule::CLOSE {
            // replace
            out = out.replace(&format!("e#{}#", block_string), "</span>");
            continue;
        }

        // e#animation (identifier) {attrs}#
        if btype == Rule::ANIMATION {
            let tag = inner.get(0).unwrap();

            let attrs = inner
                .iter()
                .skip(1)
                .into_iter()
                .collect::<Vec<&Pair<'_, Rule>>>();

            // build attrs string
            let mut attrs_string = String::new();

            for attr in attrs {
                attrs_string += &format!("{} ", attr.as_span().as_str());
            }

            // replace
            out = out.replace(
                &format!("e#{}#", block_string.replace("\"", "&quot;")),
                &format!("<span role=\"animation\" style=\"animation: {} {} ease-in-out forwards running; display: inline-block;\">", tag.as_span().as_str(), attrs_string),
            );

            continue;
        }
    }

    // only a little bit of regex-ing remains now

    // allowed elements
    let allowed_elements: Vec<&str> = Vec::from([
        "hue", "sat", "lit", "theme", "comment", "p", "span", "style", "img", "div", "a", "b", "i",
        "strong", "em", "r", "rf",
    ]);

    for element in allowed_elements {
        out = regex_replace(
            &out,
            &format!("&lt;{}&gt;", element),
            &format!("<{}>", element),
        );

        out = regex_replace(
            &out,
            &format!("&lt;{}(.*?)&gt;", element),
            &format!("<{}$1>", element),
        );

        out = regex_replace(
            &out,
            &format!("&lt;/{}&gt;", element),
            &format!("</{}>", element),
        );
    }

    // ssm
    // essentially just ssm::parse_ssm_blocks, maybe clean this up later?
    let ssm_regex = RegexBuilder::new("(ssm\\#)(?<CONTENT>.*?)\\#")
        .multi_line(true)
        .dot_matches_new_line(true)
        .build()
        .unwrap();

    for capture in ssm_regex.captures_iter(&out.clone()) {
        let content = capture.name("CONTENT").unwrap().as_str().replace("$", "#");

        // compile
        let css = crate::ssm::parse_ssm_program(content.to_string());

        // replace
        out = out.replace(
            capture.get(0).unwrap().as_str(),
            &format!("<style>{css}</style>"),
        );
    }

    // text color (bundlrs style)
    let color_regex = RegexBuilder::new("(c\\#)\\s*(?<COLOR>.*?)\\s*\\#\\s*(?<CONTENT>.*?)\\s*\\#")
        .multi_line(true)
        .dot_matches_new_line(true)
        .build()
        .unwrap();

    for capture in color_regex.captures_iter(&out.clone()) {
        let content = capture.name("CONTENT").unwrap().as_str();
        let color = capture.name("COLOR").unwrap().as_str().replace("$", "#");

        // replace
        out = out.replacen(
            capture.get(0).unwrap().as_str(),
            &format!("<span style=\"color: {color}\" role=\"custom-color\">{content}</span>"),
            1,
        );
    }

    // text color thing
    out = regex_replace_exp(
        &out,
        RegexBuilder::new(r"%(.*?)%\s*(.*?)\s*(%{2})")
            .multi_line(true)
            .dot_matches_new_line(true),
        "<span style=\"color: $1;\" role=\"custom-color\">$2</span>",
    );

    // spoiler
    out = regex_replace(
        &out,
        "(\\|\\|)\\s*(?<CONTENT>.*?)\\s*(\\|\\|)",
        "<span role=\"spoiler\">$2</span>",
    );

    out = regex_replace(
        &out,
        "(\\!\\&gt;)\\s*(?<CONTENT>.*?)($|\\s\\s)",
        "<span role=\"spoiler\">$2</span>",
    );

    // highlight
    out = regex_replace(
        &out,
        "(\\={2})(.*?)(\\={2})",
        "<span class=\"highlight\">$2</span>",
    );

    // unescape arrow alignment
    out = regex_replace(&out, "-&gt;&gt;", "->>");
    out = regex_replace(&out, "&lt;&lt;-", "<<-");

    out = regex_replace(&out, "-&gt;", "->");
    out = regex_replace(&out, "&lt;-", "<-");

    // arrow alignment (flex)
    let arrow_alignment_flex_regex = RegexBuilder::new("(\\->{2})(.*?)(\\->{2}|<{2}\\-)")
        .multi_line(true)
        .dot_matches_new_line(true)
        .build()
        .unwrap();

    for capture in arrow_alignment_flex_regex.captures_iter(&out.clone()) {
        let _match = capture.get(0).unwrap().as_str();
        let content = capture.get(2).unwrap().as_str();

        let align = if _match.ends_with(">") {
            "right"
        } else {
            "center"
        };

        out = out.replacen(
            _match,
            &format!("<rf class=\"justify-{align}\">{content}</rf>\n"),
            1,
        );
    }

    // arrow alignment
    let arrow_alignment_regex = RegexBuilder::new("(\\->{1})(.*?)(\\->{1}|<{1}\\-)")
        .multi_line(true)
        .dot_matches_new_line(true)
        .build()
        .unwrap();

    for capture in arrow_alignment_regex.captures_iter(&out.clone()) {
        let _match = capture.get(0).unwrap().as_str();
        let content = capture.get(2).unwrap().as_str();

        let align = if _match.ends_with(">") {
            "right"
        } else {
            "center"
        };

        out = out.replacen(
            _match,
            &format!("<r class=\"text-{align}\">{content}</r>\n"),
            1,
        );
    }

    // some bbcode stuff
    out = regex_replace(&out, r"\[b\](.*?)\[/b\]", "<strong>$1</strong>"); // bold
    out = regex_replace(&out, r"\[i\](.*?)\[/i\]", "<em>$1</em>"); // italic
    out = regex_replace(&out, r"\[bi\](.*?)\[/bi\]", "<strong><em>$1</em></strong>"); // bold + italic

    out = regex_replace(
        // underline
        &out,
        r"\[u\](.*?)\[/u\]",
        "<span style=\"text-decoration: underline;\" role=\"underline\">$1</span>",
    );

    out = regex_replace_dmnl(&out, r"\[c\](.*?)\[/c\]", "<r class=\"text-center\">$1</r>"); // center
    out = regex_replace_dmnl(&out, r"\[r\](.*?)\[/r\]", "<r class=\"text-right\">$1</r>"); // right

    out = regex_replace_dmnl(
        // text color
        &out,
        r"\[t (.*?)\](.*?)\[/t\]",
        "<span style=\"color: $1;\" role=\"custom-color\">$2</span>",
    );

    out = regex_replace_dmnl(
        // message
        &out,
        r"\[m (.*?)\](.*?)\[/m\]",
        "<span title=\"$1\" role=\"custom-message\">$2</span>",
    );

    out = regex_replace(
        // highlight
        &out,
        r"\[h\](.*?)\[/h\]",
        "<span class=\"highlight\">$1</span>",
    );

    out = regex_replace(
        // highlight
        &out,
        r"\[h\](.*?)\[/h\]",
        "<span class=\"highlight\">$1</span>",
    );

    for i in 1..7 {
        // headings
        out = regex_replace(
            &out,
            &format!(r"\[h{i}\](.*?)\[/h{i}\]"),
            &format!("<h{i} id=\"$1\">$1</h{i}>"),
        );
    }

    out = regex_replace(&out, r"\[/\]", "<br />"); // line break

    out = regex_replace(
        // code
        &out,
        r"\[cl\](.*?)\[/cl\]",
        "<code>$1</code>",
    );

    out = regex_replace_dmnl(
        // fenced code
        &out,
        r"\[src (.*?)\]\n(.*?)\n\[/src\]",
        "<pre class=\"lang-$1\"><code>$2</code></pre>",
    );

    // bath time
    out = regex_replace(&out, "^(on)(.*)\\=(.*)\"$", "");
    out = regex_replace(&out, "(href)\\=\"(javascript\\:)(.*)\"", "");

    out = regex_replace(&out, "(<script.*>)(.*?)(<\\/script>)", "");
    out = regex_replace(&out, "(<script.*>)", "");
    out = regex_replace(&out, "(<link.*>)", "");
    out = regex_replace(&out, "(<meta.*>)", "");

    // return
    out
}

pub fn parse_markdown(input: &str) -> String {
    let tree = super::parser::parse(input);
    from_tree(&tree.into_inner(), input.to_owned())
}

fn regex_replace(input: &str, pattern: &str, replace_with: &str) -> String {
    RegexBuilder::new(pattern)
        .multi_line(true)
        .build()
        .unwrap()
        .replace_all(input, replace_with)
        .to_string()
}

fn regex_replace_dmnl(input: &str, pattern: &str, replace_with: &str) -> String {
    RegexBuilder::new(pattern)
        .multi_line(true)
        .dot_matches_new_line(true)
        .build()
        .unwrap()
        .replace_all(input, replace_with)
        .to_string()
}

#[allow(dead_code)]
fn regex_replace_exp(input: &str, pattern: &mut RegexBuilder, replace_with: &str) -> String {
    pattern
        .build()
        .unwrap()
        .replace_all(input, replace_with)
        .to_string()
}
