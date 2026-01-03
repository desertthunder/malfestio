#!/bin/bash
mkdir -p crates/readability/tests/data

fetch_if_missing() {
    local url="$1"
    local output="$2"

    if [ -f "$output" ]; then
        echo "Cached: $output"
    else
        echo "Fetching $url..."
        curl -L -H "User-Agent: Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36" -o "$output" "$url"
    fi
}

fetch_if_missing "https://arxiv.org/abs/2009.03017" "crates/readability/tests/data/arxiv.html"
fetch_if_missing "https://en.wikipedia.org/wiki/Rust_(programming_language)" "crates/readability/tests/data/wikipedia.html"
fetch_if_missing "https://dougshapiro.medium.com/how-will-the-disruption-of-hollywood-play-out-42f724c921e1" "crates/readability/tests/data/medium.html"
fetch_if_missing "https://taibbi.substack.com/p/glenn-greenwald-on-his-resignation" "crates/readability/tests/data/substack.html"
fetch_if_missing "https://www.theonion.com/theresa-may-narrowly-manages-to-survive-parliamentary-f-1831077604" "crates/readability/tests/data/theonion.html"
fetch_if_missing "http://docs.readthedocs.io/en/latest/getting_started.html" "crates/readability/tests/data/readthedocs.html"
fetch_if_missing "https://www.rust-lang.org/" "crates/readability/tests/data/generic.html"

echo "Done."
