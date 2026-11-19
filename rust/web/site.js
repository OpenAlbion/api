// OpenAlbion site interactions: mobile navigation, copy buttons, code tabs,
//! the live API playground and the documentation filter. No dependencies,
//! no tracking, no third-party requests.

(function () {
    "use strict";

    document.documentElement.classList.add("js");

    // Keep the docs menu collapsed by default on small screens.
    var docsSidebar = document.querySelector(".docs-sidebar");
    if (docsSidebar) {
        var docsMedia = window.matchMedia("(max-width: 900px)");
        if (docsMedia.matches) {
            docsSidebar.removeAttribute("open");
        }
        var openDocsOnDesktop = function () {
            if (!docsMedia.matches) {
                docsSidebar.setAttribute("open", "");
            }
        };
        if (docsMedia.addEventListener) {
            docsMedia.addEventListener("change", openDocsOnDesktop);
        } else if (docsMedia.addListener) {
            docsMedia.addListener(openDocsOnDesktop);
        }
    }

    /* Mobile navigation ---------------------------------------------------- */

    var header = document.querySelector(".site-header");
    var navToggle = document.querySelector(".nav-toggle");

    if (header && navToggle) {
        navToggle.addEventListener("click", function () {
            var open = header.classList.toggle("nav-open");
            navToggle.setAttribute("aria-expanded", open ? "true" : "false");
        });

        header.querySelectorAll(".site-nav a").forEach(function (link) {
            link.addEventListener("click", function () {
                header.classList.remove("nav-open");
                navToggle.setAttribute("aria-expanded", "false");
            });
        });

        document.addEventListener("keydown", function (event) {
            if (event.key === "Escape" && header.classList.contains("nav-open")) {
                header.classList.remove("nav-open");
                navToggle.setAttribute("aria-expanded", "false");
                navToggle.focus();
            }
        });
    }

    /* Copy helpers --------------------------------------------------------- */

    function fallbackCopy(text) {
        var area = document.createElement("textarea");
        area.value = text;
        area.setAttribute("readonly", "");
        area.style.position = "fixed";
        area.style.top = "-1000px";
        document.body.appendChild(area);
        area.select();
        try {
            document.execCommand("copy");
        } catch (error) {
            /* ignore: the user can still select the text manually */
        }
        document.body.removeChild(area);
    }

    function flash(button) {
        if (!button) {
            return;
        }
        var label = button.querySelector("span");
        var original = label ? label.textContent : null;
        button.classList.add("is-copied");
        if (label) {
            label.textContent = "Copied!";
        }
        window.setTimeout(function () {
            button.classList.remove("is-copied");
            if (label && original !== null) {
                label.textContent = original;
            }
        }, 1600);
    }

    function copyText(text, button) {
        if (!text) {
            return;
        }
        if (navigator.clipboard && window.isSecureContext) {
            navigator.clipboard.writeText(text).then(function () {
                flash(button);
            }).catch(function () {
                fallbackCopy(text);
                flash(button);
            });
        } else {
            fallbackCopy(text);
            flash(button);
        }
    }

    document.querySelectorAll("[data-copy]").forEach(function (button) {
        button.addEventListener("click", function () {
            copyText(button.getAttribute("data-copy") || "", button);
        });
    });

    document.querySelectorAll("[data-copy-target]").forEach(function (button) {
        button.addEventListener("click", function () {
            var target = document.getElementById(button.getAttribute("data-copy-target"));
            copyText(target ? target.textContent : "", button);
        });
    });

    /* Code tabs ------------------------------------------------------------ */

    document.querySelectorAll("[data-tabs]").forEach(function (tabs) {
        var buttons = tabs.querySelectorAll("[data-tab]");
        var panels = tabs.querySelectorAll("[data-panel]");

        function select(name) {
            buttons.forEach(function (button) {
                var active = button.getAttribute("data-tab") === name;
                button.setAttribute("aria-selected", active ? "true" : "false");
                button.classList.toggle("is-active", active);
            });
            panels.forEach(function (panel) {
                panel.hidden = panel.getAttribute("data-panel") !== name;
            });
        }

        buttons.forEach(function (button) {
            button.addEventListener("click", function () {
                select(button.getAttribute("data-tab"));
            });
        });

        var copyButton = tabs.querySelector("[data-copy-panel]");
        if (copyButton) {
            copyButton.addEventListener("click", function () {
                var visible = tabs.querySelector("[data-panel]:not([hidden]) code");
                copyText(visible ? visible.textContent : "", copyButton);
            });
        }
    });

    /* Documentation filter ------------------------------------------------- */

    var filter = document.getElementById("doc-search");
    if (filter) {
        var sidebar = filter.closest(".docs-sidebar-inner") || document;
        var links = Array.prototype.slice.call(sidebar.querySelectorAll(".docs-nav-link"));
        var lists = Array.prototype.slice.call(sidebar.querySelectorAll(".docs-nav-list"));
        var empty = sidebar.querySelector(".docs-filter-empty");

        filter.addEventListener("input", function () {
            var query = filter.value.trim().toLowerCase();
            var visible = 0;

            links.forEach(function (link) {
                var haystack = (link.getAttribute("data-search") || "") + " " + link.textContent;
                var match = query === "" || haystack.toLowerCase().indexOf(query) !== -1;
                link.parentElement.hidden = !match;
                if (match) {
                    visible += 1;
                }
            });

            lists.forEach(function (list) {
                var hasVisible = Array.prototype.some.call(
                    list.querySelectorAll("li"),
                    function (item) {
                        return !item.hidden;
                    }
                );
                list.hidden = query !== "" && !hasVisible;
                var title = list.previousElementSibling;
                if (title && title.classList.contains("docs-nav-title")) {
                    title.hidden = query !== "" && !hasVisible;
                }
            });

            if (empty) {
                empty.hidden = visible !== 0;
            }
        });
    }

    /* Playground ----------------------------------------------------------- */

    var playground = document.querySelector("[data-playground]");
    if (!playground) {
        return;
    }

    var form = playground.querySelector("[data-playground-form]");
    var input = playground.querySelector("#pg-path");
    var output = playground.querySelector("[data-playground-output]");
    var status = playground.querySelector("[data-playground-status]");
    var runButton = playground.querySelector(".btn-run");
    var presets = playground.querySelectorAll("[data-endpoint]");
    var activeRequest = null;

    function setStatus(text, kind) {
        if (!status) {
            return;
        }
        status.textContent = text;
        status.classList.remove("is-ok", "is-error", "is-loading");
        if (kind) {
            status.classList.add(kind);
        }
    }

    function humanSize(bytes) {
        if (bytes < 1024) {
            return bytes + " B";
        }
        if (bytes < 1024 * 1024) {
            return (bytes / 1024).toFixed(1) + " KB";
        }
        return (bytes / (1024 * 1024)).toFixed(1) + " MB";
    }

    function safePath(value) {
        var path = (value || "").trim();
        if (!path.startsWith("/api/")) {
            throw new Error("The path must start with /api/.");
        }
        var url = new URL(path, window.location.origin);
        if (url.origin !== window.location.origin || !url.pathname.startsWith("/api/")) {
            throw new Error("Only requests to this API are allowed.");
        }
        return url.pathname + url.search;
    }

    function render(data, elapsed, bytes, statusCode, statusText) {
        statusText = statusText || "OK";
        var text;
        try {
            text = JSON.stringify(JSON.parse(data), null, 2);
        } catch (error) {
            text = data;
        }
        var truncated = "";
        if (text.length > 120000) {
            text = text.slice(0, 120000);
            truncated = "\n\n… output truncated at 120 KB …";
        }
        if (output) {
            output.textContent = text + truncated;
        }
        setStatus(statusCode + " " + statusText + " · " + elapsed + " ms · " + humanSize(bytes), "is-ok");
    }

    function runRequest(path) {
        if (activeRequest) {
            activeRequest.abort();
        }
        var controller = new AbortController();
        activeRequest = controller;
        var started = performance.now();
        var timeout = window.setTimeout(function () {
            controller.abort();
        }, 20000);

        if (runButton) {
            runButton.disabled = true;
        }
        setStatus("Loading…", "is-loading");
        if (output) {
            output.textContent = "Loading " + path + " …";
        }

        fetch(path, {
            method: "GET",
            headers: { Accept: "application/json" },
            signal: controller.signal,
        })
            .then(function (response) {
                return response.text().then(function (body) {
                    return { response: response, body: body };
                });
            })
            .then(function (result) {
                var elapsed = Math.round(performance.now() - started);
                var bytes = new Blob([result.body]).size;
                var response = result.response;
                if (!response.ok) {
                    var retry = response.headers.get("retry-after");
                    var suffix = retry ? " · retry in " + retry + "s" : "";
                    setStatus(response.status + " " + response.statusText + suffix, "is-error");
                    if (output) {
                        output.textContent = result.body || response.statusText;
                    }
                    return;
                }
                render(result.body, elapsed, bytes, response.status, response.statusText);
            })
            .catch(function (error) {
                if (error && error.name === "AbortError") {
                    setStatus("Request cancelled", "is-error");
                } else {
                    setStatus("Request failed", "is-error");
                }
                if (output) {
                    output.textContent = "The request could not be completed.\n\n" + (error && error.message ? error.message : String(error));
                }
            })
            .finally(function () {
                window.clearTimeout(timeout);
                if (runButton) {
                    runButton.disabled = false;
                }
                if (activeRequest === controller) {
                    activeRequest = null;
                }
            });
    }

    if (form && input) {
        form.addEventListener("submit", function (event) {
            event.preventDefault();
            try {
                runRequest(safePath(input.value));
            } catch (error) {
                setStatus("Check the path", "is-error");
                if (output) {
                    output.textContent = error.message;
                }
            }
        });
    }

    presets.forEach(function (button) {
        button.addEventListener("click", function () {
            var path = button.getAttribute("data-endpoint") || "";
            presets.forEach(function (other) {
                other.classList.toggle("is-active", other === button);
            });
            if (input) {
                input.value = path;
            }
            try {
                runRequest(safePath(path));
            } catch (error) {
                setStatus("Check the path", "is-error");
            }
        });
    });
})();
