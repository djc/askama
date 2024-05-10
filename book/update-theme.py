#!/usr/bin/env python3

from base64 import b64decode
from contextlib import closing
from http.client import HTTPSConnection
from io import StringIO
from json import load
from os import PathLike
from pathlib import Path


INDEX_HBS_DOMAIN = "api.github.com"
INDEX_HBS_PORT = 443
INDEX_HBS_PROTO = "GET"
INDEX_HBS_PATH = "/repos/rust-lang/mdBook/contents/src/theme/index.hbs"
INDEX_HBS_HEADERS = {
    "user-agent": "Update index.hbs for +https://github.com/djc/askama",
}

SIDEBAR_STYLE = 'style="display:flex; flex-direction:column"'
SCROLLBOX_END = r"""
<div id="ethical-ad-placement" data-ea-publisher="readthedocs" data-ea-type="image"></div>
<readthedocs-flyout></readthedocs-flyout>
"""
TOC_START = '<div style="flex:1">'
TOC_END = "</div>"
SIDEBAR_END = r"""
<script>
    document.addEventListener("DOMContentLoaded", function insertStyle () {
        const elem = customElements.get("readthedocs-flyout");
        if (elem) {
            elem.styles.insertRule(`
                .container {
                    position: unset !important;
                    max-width: unset !important;
                    width: unset !important;
                    height: unset !important;
                    max-height: unset !important;
                }
            `);
            elem.styles.insertRule(`
                dl:has(#flyout-search-form) {
                    display: none !important;
                }
            `);
        } else {
            setTimeout(insertStyle, 50);
        }
    });
</script>
"""


def update_theme(target: PathLike) -> None:
    with closing(HTTPSConnection(INDEX_HBS_DOMAIN, INDEX_HBS_PORT)) as conn:
        conn.request(INDEX_HBS_PROTO, INDEX_HBS_PATH, None, INDEX_HBS_HEADERS)
        res = conn.getresponse()
        if res.status != 200:
            raise Exception(f"Status={res.status!r}")
        data = load(res)

    if data["encoding"] != "base64":
        raise Exception(f'Encoding={data["encoding"]!r}')

    input_f = StringIO(str(b64decode(data["content"]), "UTF-8"))
    output_f = StringIO()

    _, revision = data["git_url"].rsplit("/", 1)
    print("{{!-- Source revision:", revision, "--}}", file=output_f)

    state = "before-sidebar"
    for line in input_f:
        match state:
            case "before-sidebar" if '<nav id="sidebar"' in line:
                state = "before-scrollbox"
            case "before-scrollbox" if '<div class="sidebar-scrollbox"' in line:
                line = line[:-2]  # remove '>\n'
                line = f"{line} {SIDEBAR_STYLE}>\n"
                state = "before-toc"
            case "before-toc" if "{{#toc}}{{/toc}}" in line:
                indent = line[: len(line) - len(line.lstrip())]
                line = f"{indent}{TOC_START}{line.strip()}{TOC_END}\n"
                state = "in-scrollbox"
            case "in-scrollbox" if line.strip() == "</div>":
                indent = line[: len(line) - len(line.lstrip())]
                for s in SCROLLBOX_END.splitlines():
                    if s:
                        print(indent, s, sep="    ", file=output_f)
                state = "in-sidebar"
            case "in-sidebar" if line.strip() == "</nav>":
                indent = line[: len(line) - len(line.lstrip())]
                for s in SIDEBAR_END.splitlines():
                    if s:
                        print(indent, s, sep="    ", file=output_f)
                state = "after-sidebar"
        output_f.write(line)

    output_f.seek(0, 0)
    with open(target, "wt") as f:
        print(output_f.read(), end="", file=f)


if __name__ == "__main__":
    update_theme(Path(__file__).absolute().parent / "theme" / "index.hbs")
